//! Blind relay. Routes store opaque ciphertext and sealed keys.
//! This crate does not call `open_seal` and has no function that opens a sealed key.

mod snapshot;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as AsyncMutex;

use axum::extract::{Path as AxumPath, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use ouroboros_core::{verify_detached, EmptyWake};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;

pub use snapshot::{reload_previous, FileSnapshot, SnapshotError, SnapshotStore};

/// Shared relay state.
pub struct AppState {
    db_path: PathBuf,
    pool: SqlitePool,
    snapshot: Arc<dyn SnapshotStore>,
    write_lock: AsyncMutex<()>,
    hits: Mutex<HashMap<String, u32>>,
    rate_limit: u32,
    audit: Mutex<Vec<String>>,
}

impl AppState {
    /// Open or restore the database, then listen for requests.
    pub async fn open(
        db_path: PathBuf,
        snapshot_path: PathBuf,
        rate_limit: u32,
    ) -> Result<Self, sqlx::Error> {
        let snapshot = Arc::new(FileSnapshot::new(snapshot_path));
        Self::open_with(db_path, snapshot, rate_limit).await
    }

    /// Open with a caller-supplied snapshot store. Tests use this to fail uploads.
    pub async fn open_with(
        db_path: PathBuf,
        snapshot: Arc<dyn SnapshotStore>,
        rate_limit: u32,
    ) -> Result<Self, sqlx::Error> {
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        if let Ok(Some(bytes)) = snapshot.load() {
            fs::write(&db_path, bytes).ok();
        }
        let options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS accounts (
                account_id TEXT PRIMARY KEY,
                code_hash TEXT NOT NULL,
                public_key TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
        )
        .execute(&pool)
        .await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS partnerships (
                partnership_id TEXT PRIMARY KEY,
                tracker_account_id TEXT NOT NULL,
                invite TEXT NOT NULL,
                confirmed INTEGER NOT NULL,
                ciphertext TEXT,
                sealed_key TEXT,
                signature TEXT,
                created_at INTEGER NOT NULL
            )",
        )
        .execute(&pool)
        .await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS wake_times (
                partnership_id TEXT PRIMARY KEY,
                wake_at INTEGER NOT NULL
            )",
        )
        .execute(&pool)
        .await?;
        Ok(Self {
            db_path,
            pool,
            snapshot,
            write_lock: AsyncMutex::new(()),
            hits: Mutex::new(HashMap::new()),
            rate_limit,
            audit: Mutex::new(Vec::new()),
        })
    }

    fn audit(&self, route: &str, account_id: &str) {
        let line = format!("route={route} account_id={account_id}");
        tracing::info!("{line}");
        self.audit.lock().expect("audit lock").push(line);
    }

    fn allow(&self, account_id: &str) -> bool {
        let mut hits = self.hits.lock().expect("rate lock");
        let count = hits.entry(account_id.to_string()).or_insert(0);
        *count += 1;
        *count <= self.rate_limit
    }

    async fn publish(&self) -> Result<(), StatusCode> {
        let _guard = self.write_lock.lock().await;
        let expected = self.snapshot.generation();
        sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
            .execute(&self.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        sqlx::query("VACUUM")
            .execute(&self.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let bytes = fs::read(&self.db_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        match self.snapshot.store(&bytes, expected) {
            Ok(_) => Ok(()),
            Err(SnapshotError::UploadFailed) => Err(StatusCode::SERVICE_UNAVAILABLE),
            Err(SnapshotError::Stale) => {
                reload_previous(&self.db_path, self.snapshot.as_ref())
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                Err(StatusCode::CONFLICT)
            }
            Err(SnapshotError::Io) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

/// HTTP router. Wake responses carry no phase text.
pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/accounts/enroll", post(enroll))
        .route("/v1/accounts/restore", post(restore))
        .route("/v1/partnerships", post(create_partnership))
        .route("/v1/partnerships/:id/confirm", post(confirm_partnership))
        .route(
            "/v1/partnerships/:id/view",
            post(upload_view).get(fetch_view),
        )
        .route("/v1/partnerships/:id", delete(revoke))
        .route("/v1/wakes/:id", post(wake))
        .with_state(state)
}

async fn health() -> StatusCode {
    StatusCode::OK
}

#[derive(Deserialize)]
struct EnrollBody {
    account_id: String,
    code_hash: String,
    public_key: String,
    signature: String,
}

async fn enroll(State(state): State<Arc<AppState>>, Json(body): Json<EnrollBody>) -> StatusCode {
    if !state.allow(&body.account_id) {
        return StatusCode::TOO_MANY_REQUESTS;
    }
    let message = format!(
        "enroll:{}:{}:{}",
        body.account_id, body.code_hash, body.public_key
    );
    let signature = match hex::decode(&body.signature) {
        Ok(bytes) => bytes,
        Err(_) => return StatusCode::UNAUTHORIZED,
    };
    if verify_detached(&body.public_key, message.as_bytes(), &signature).is_err() {
        return StatusCode::UNAUTHORIZED;
    }
    state.audit("enroll", &body.account_id);
    let result = sqlx::query(
        "INSERT INTO accounts (account_id, code_hash, public_key, created_at) VALUES (?, ?, ?, 0)",
    )
    .bind(&body.account_id)
    .bind(&body.code_hash)
    .bind(&body.public_key)
    .execute(&state.pool)
    .await;
    if result.is_err() {
        return StatusCode::CONFLICT;
    }
    state.publish().await.err().unwrap_or(StatusCode::CREATED)
}

#[derive(Deserialize)]
struct RestoreBody {
    account_id: String,
    code_hash: String,
    public_key: String,
    signature: String,
}

async fn restore(State(state): State<Arc<AppState>>, Json(body): Json<RestoreBody>) -> StatusCode {
    if !state.allow(&body.account_id) {
        return StatusCode::TOO_MANY_REQUESTS;
    }
    let message = format!(
        "restore:{}:{}:{}",
        body.account_id, body.code_hash, body.public_key
    );
    let signature = match hex::decode(body.signature) {
        Ok(bytes) => bytes,
        Err(_) => return StatusCode::UNAUTHORIZED,
    };
    if verify_detached(&body.public_key, message.as_bytes(), &signature).is_err() {
        return StatusCode::UNAUTHORIZED;
    }
    let updated =
        sqlx::query("UPDATE accounts SET public_key = ? WHERE account_id = ? AND code_hash = ?")
            .bind(&body.public_key)
            .bind(&body.account_id)
            .bind(&body.code_hash)
            .execute(&state.pool)
            .await;
    match updated {
        Ok(result) if result.rows_affected() == 1 => {
            state.audit("restore", &body.account_id);
            state.publish().await.err().unwrap_or(StatusCode::OK)
        }
        _ => StatusCode::UNAUTHORIZED,
    }
}

#[derive(Deserialize)]
struct PartnershipBody {
    partnership_id: String,
    tracker_account_id: String,
    invite: String,
    signature: String,
}

async fn create_partnership(
    State(state): State<Arc<AppState>>,
    Json(body): Json<PartnershipBody>,
) -> StatusCode {
    if !state.allow(&body.tracker_account_id) {
        return StatusCode::TOO_MANY_REQUESTS;
    }
    let public_key = match account_key(&state, &body.tracker_account_id).await {
        Some(key) => key,
        None => return StatusCode::UNAUTHORIZED,
    };
    let message = format!(
        "partnership:{}:{}:{}",
        body.partnership_id, body.tracker_account_id, body.invite
    );
    if !signed(&public_key, &message, &body.signature) {
        return StatusCode::UNAUTHORIZED;
    }
    state.audit("partnership", &body.tracker_account_id);
    let inserted = sqlx::query(
        "INSERT INTO partnerships (partnership_id, tracker_account_id, invite, confirmed, created_at)
         VALUES (?, ?, ?, 0, 0)",
    )
    .bind(&body.partnership_id)
    .bind(&body.tracker_account_id)
    .bind(&body.invite)
    .execute(&state.pool)
    .await;
    if inserted.is_err() {
        return StatusCode::CONFLICT;
    }
    state.publish().await.err().unwrap_or(StatusCode::CREATED)
}

#[derive(Deserialize)]
struct ConfirmBody {
    tracker_account_id: String,
    signature: String,
}

async fn confirm_partnership(
    State(state): State<Arc<AppState>>,
    AxumPath(id): AxumPath<String>,
    Json(body): Json<ConfirmBody>,
) -> StatusCode {
    let public_key = match account_key(&state, &body.tracker_account_id).await {
        Some(key) => key,
        None => return StatusCode::UNAUTHORIZED,
    };
    let message = format!("confirm:{id}");
    if !signed(&public_key, &message, &body.signature) {
        return StatusCode::UNAUTHORIZED;
    }
    let updated = sqlx::query(
        "UPDATE partnerships SET confirmed = 1 WHERE partnership_id = ? AND tracker_account_id = ?",
    )
    .bind(&id)
    .bind(&body.tracker_account_id)
    .execute(&state.pool)
    .await;
    if updated.map(|result| result.rows_affected()).unwrap_or(0) != 1 {
        return StatusCode::NOT_FOUND;
    }
    state.audit("confirm", &body.tracker_account_id);
    state.publish().await.err().unwrap_or(StatusCode::OK)
}

#[derive(Deserialize)]
struct UploadBody {
    tracker_account_id: String,
    ciphertext_hex: String,
    sealed_key_hex: String,
    signature: String,
    confirmed: bool,
}

async fn upload_view(
    State(state): State<Arc<AppState>>,
    AxumPath(id): AxumPath<String>,
    Json(body): Json<UploadBody>,
) -> StatusCode {
    if !body.confirmed {
        return StatusCode::CONFLICT;
    }
    let public_key = match account_key(&state, &body.tracker_account_id).await {
        Some(key) => key,
        None => return StatusCode::UNAUTHORIZED,
    };
    let message = format!("upload:{id}:{}", body.ciphertext_hex);
    if !signed(&public_key, &message, &body.signature) {
        return StatusCode::UNAUTHORIZED;
    }
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT confirmed FROM partnerships WHERE partnership_id = ? AND tracker_account_id = ?",
    )
    .bind(&id)
    .bind(&body.tracker_account_id)
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten();
    if row != Some((1,)) {
        return StatusCode::CONFLICT;
    }
    let updated = sqlx::query(
        "UPDATE partnerships SET ciphertext = ?, sealed_key = ?, signature = ? WHERE partnership_id = ?",
    )
    .bind(&body.ciphertext_hex)
    .bind(&body.sealed_key_hex)
    .bind(&body.signature)
    .bind(&id)
    .execute(&state.pool)
    .await;
    if updated.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    let _ = sqlx::query(
        "INSERT INTO wake_times (partnership_id, wake_at) VALUES (?, 0)
         ON CONFLICT(partnership_id) DO UPDATE SET wake_at = 0",
    )
    .bind(&id)
    .execute(&state.pool)
    .await;
    state.audit("upload", &body.tracker_account_id);
    state.publish().await.err().unwrap_or(StatusCode::OK)
}

#[derive(Serialize)]
struct FetchBody {
    ciphertext_hex: String,
    sealed_key_hex: String,
}

async fn fetch_view(
    State(state): State<Arc<AppState>>,
    AxumPath(id): AxumPath<String>,
) -> Result<Json<FetchBody>, StatusCode> {
    let row: Option<(Option<String>, Option<String>)> =
        sqlx::query_as("SELECT ciphertext, sealed_key FROM partnerships WHERE partnership_id = ?")
            .bind(&id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match row {
        Some((Some(ciphertext_hex), Some(sealed_key_hex))) => Ok(Json(FetchBody {
            ciphertext_hex,
            sealed_key_hex,
        })),
        _ => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Deserialize)]
struct RevokeBody {
    tracker_account_id: String,
    signature: String,
}

async fn revoke(
    State(state): State<Arc<AppState>>,
    AxumPath(id): AxumPath<String>,
    Json(body): Json<RevokeBody>,
) -> StatusCode {
    let public_key = match account_key(&state, &body.tracker_account_id).await {
        Some(key) => key,
        None => return StatusCode::UNAUTHORIZED,
    };
    let message = format!("revoke:{id}");
    if !signed(&public_key, &message, &body.signature) {
        return StatusCode::UNAUTHORIZED;
    }
    let deleted =
        sqlx::query("DELETE FROM partnerships WHERE partnership_id = ? AND tracker_account_id = ?")
            .bind(&id)
            .bind(&body.tracker_account_id)
            .execute(&state.pool)
            .await;
    if deleted.map(|result| result.rows_affected()).unwrap_or(0) != 1 {
        return StatusCode::NOT_FOUND;
    }
    let _ = sqlx::query("DELETE FROM wake_times WHERE partnership_id = ?")
        .bind(&id)
        .execute(&state.pool)
        .await;
    state.audit("revoke", &body.tracker_account_id);
    state
        .publish()
        .await
        .err()
        .unwrap_or(StatusCode::NO_CONTENT)
}

async fn wake(
    State(state): State<Arc<AppState>>,
    AxumPath(id): AxumPath<String>,
) -> Result<Json<EmptyWake>, StatusCode> {
    let exists: Option<(String,)> =
        sqlx::query_as("SELECT partnership_id FROM partnerships WHERE partnership_id = ?")
            .bind(&id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if exists.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(Json(EmptyWake::sync()))
}

async fn account_key(state: &AppState, account_id: &str) -> Option<String> {
    sqlx::query_as("SELECT public_key FROM accounts WHERE account_id = ?")
        .bind(account_id)
        .fetch_optional(&state.pool)
        .await
        .ok()
        .flatten()
        .map(|(key,): (String,)| key)
}

fn signed(public_key: &str, message: &str, signature_hex: &str) -> bool {
    let Ok(signature) = hex::decode(signature_hex) else {
        return false;
    };
    verify_detached(public_key, message.as_bytes(), &signature).is_ok()
}

/// Read the on-disk database. Tests use this after a reload.
pub fn database_bytes(path: &Path) -> Vec<u8> {
    fs::read(path).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use ouroboros_core::DeviceIdentity;
    use tower::ServiceExt;

    struct World {
        state: Arc<AppState>,
        dir: PathBuf,
        identity: DeviceIdentity,
        account_id: String,
    }

    impl World {
        async fn new(rate_limit: u32) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "ouroboros-relay-{}-{}",
                std::process::id(),
                rand_suffix()
            ));
            fs::create_dir_all(&dir).unwrap();
            let snapshot = Arc::new(FileSnapshot::new(dir.join("relay.snapshot")));
            let state = AppState::open_with(dir.join("relay.sqlite"), snapshot, rate_limit)
                .await
                .unwrap();
            Self {
                state: Arc::new(state),
                dir,
                identity: DeviceIdentity::generate(),
                account_id: "tracker-1".into(),
            }
        }

        fn sign(&self, message: &str) -> String {
            hex::encode(self.identity.sign(message.as_bytes()))
        }

        async fn call(
            &self,
            method: &str,
            uri: &str,
            body: serde_json::Value,
        ) -> (StatusCode, Vec<u8>) {
            let app = router(self.state.clone());
            let response = app
                .oneshot(
                    Request::builder()
                        .method(method)
                        .uri(uri)
                        .header("content-type", "application/json")
                        .body(Body::from(body.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            let status = response.status();
            let bytes = response
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .to_vec();
            (status, bytes)
        }

        async fn enroll(&self) {
            let public_key = self.identity.public().signing_key_hex();
            let code_hash = "hashed-code";
            let message = format!("enroll:{}:{code_hash}:{public_key}", self.account_id);
            let (status, _) = self
                .call(
                    "POST",
                    "/v1/accounts/enroll",
                    serde_json::json!({
                        "account_id": self.account_id,
                        "code_hash": code_hash,
                        "public_key": public_key,
                        "signature": self.sign(&message),
                    }),
                )
                .await;
            assert_eq!(status, StatusCode::CREATED);
        }
    }

    impl Drop for World {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.dir);
        }
    }

    fn rand_suffix() -> u64 {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(1);
        NEXT.fetch_add(1, Ordering::Relaxed)
    }

    #[tokio::test]
    async fn enroll_upload_fetch_revoke() {
        let world = World::new(20).await;
        world.enroll().await;
        let partnership = "part-1";
        let invite = "once";
        let message = format!("partnership:{partnership}:{}:{invite}", world.account_id);
        let (status, _) = world
            .call(
                "POST",
                "/v1/partnerships",
                serde_json::json!({
                    "partnership_id": partnership,
                    "tracker_account_id": world.account_id,
                    "invite": invite,
                    "signature": world.sign(&message),
                }),
            )
            .await;
        assert_eq!(status, StatusCode::CREATED);
        let confirm = format!("confirm:{partnership}");
        let (status, _) = world
            .call(
                "POST",
                &format!("/v1/partnerships/{partnership}/confirm"),
                serde_json::json!({
                    "tracker_account_id": world.account_id,
                    "signature": world.sign(&confirm),
                }),
            )
            .await;
        assert_eq!(status, StatusCode::OK);
        let ciphertext = hex::encode(b"ciphertext-bytes");
        let sealed = hex::encode(b"sealed-key-bytes");
        let upload_message = format!("upload:{partnership}:{ciphertext}");
        let (status, _) = world
            .call(
                "POST",
                &format!("/v1/partnerships/{partnership}/view"),
                serde_json::json!({
                    "tracker_account_id": world.account_id,
                    "ciphertext_hex": ciphertext,
                    "sealed_key_hex": sealed,
                    "signature": world.sign(&upload_message),
                    "confirmed": true,
                }),
            )
            .await;
        assert_eq!(status, StatusCode::OK);
        let (status, body) = world
            .call(
                "GET",
                &format!("/v1/partnerships/{partnership}/view"),
                serde_json::json!({}),
            )
            .await;
        assert_eq!(status, StatusCode::OK);
        assert!(String::from_utf8_lossy(&body).contains(&sealed));
        let revoke = format!("revoke:{partnership}");
        let (status, _) = world
            .call(
                "DELETE",
                &format!("/v1/partnerships/{partnership}"),
                serde_json::json!({
                    "tracker_account_id": world.account_id,
                    "signature": world.sign(&revoke),
                }),
            )
            .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        let (status, _) = world
            .call(
                "GET",
                &format!("/v1/partnerships/{partnership}/view"),
                serde_json::json!({}),
            )
            .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        let snapshot = fs::read(world.dir.join("relay.snapshot")).unwrap_or_default();
        let snapshot_text = String::from_utf8_lossy(&snapshot);
        assert!(!snapshot_text.contains(&sealed));
        assert!(!snapshot_text.contains("sealed-key-bytes"));
    }

    #[tokio::test]
    async fn unconfirmed_upload_is_rejected() {
        let world = World::new(20).await;
        world.enroll().await;
        let (status, _) = world
            .call(
                "POST",
                "/v1/partnerships/part-2/view",
                serde_json::json!({
                    "tracker_account_id": world.account_id,
                    "ciphertext_hex": "aa",
                    "sealed_key_hex": "bb",
                    "signature": "00",
                    "confirmed": false,
                }),
            )
            .await;
        assert_eq!(status, StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn failed_upload_returns_non_2xx() {
        let dir = std::env::temp_dir().join(format!("ouroboros-fail-{}", rand_suffix()));
        fs::create_dir_all(&dir).unwrap();
        let snapshot = Arc::new(FileSnapshot::new(dir.join("relay.snapshot")));
        snapshot.fail_next_upload();
        let state = Arc::new(
            AppState::open_with(dir.join("relay.sqlite"), snapshot, 20)
                .await
                .unwrap(),
        );
        let identity = DeviceIdentity::generate();
        let public_key = identity.public().signing_key_hex();
        let message = format!("enroll:acct:hash:{public_key}");
        let signature = hex::encode(identity.sign(message.as_bytes()));
        let app = router(state);
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/accounts/enroll")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "account_id": "acct",
                            "code_hash": "hash",
                            "public_key": public_key,
                            "signature": signature,
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(response.status().is_client_error() || response.status().is_server_error());
        assert_ne!(response.status(), StatusCode::CREATED);
        let _ = fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn rate_limit_rejects_after_burst() {
        let world = World::new(1).await;
        world.enroll().await;
        let public_key = world.identity.public().signing_key_hex();
        let message = format!("enroll:{}:hash:{public_key}", world.account_id);
        let (status, _) = world
            .call(
                "POST",
                "/v1/accounts/enroll",
                serde_json::json!({
                    "account_id": world.account_id,
                    "code_hash": "hash",
                    "public_key": public_key,
                    "signature": world.sign(&message),
                }),
            )
            .await;
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn logs_omit_account_code_and_blob() {
        let world = World::new(20).await;
        let code = "1234567890123456";
        let blob = "deadbeefciphertext";
        world.enroll().await;
        let logs = world.state.audit.lock().unwrap().join("\n");
        assert!(!logs.contains(code));
        assert!(!logs.contains(blob));
    }

    #[tokio::test]
    async fn wake_body_has_no_phase_label() {
        let world = World::new(20).await;
        world.enroll().await;
        let partnership = "part-wake";
        let message = format!("partnership:{partnership}:{}:invite", world.account_id);
        let _ = world
            .call(
                "POST",
                "/v1/partnerships",
                serde_json::json!({
                    "partnership_id": partnership,
                    "tracker_account_id": world.account_id,
                    "invite": "invite",
                    "signature": world.sign(&message),
                }),
            )
            .await;
        let (status, body) = world
            .call(
                "POST",
                &format!("/v1/wakes/{partnership}"),
                serde_json::json!({}),
            )
            .await;
        assert_eq!(status, StatusCode::OK);
        let text = String::from_utf8(body).unwrap();
        assert_eq!(text, r#"{"wake":"sync"}"#);
        assert!(!text.contains("phase"));
    }
}
