use std::path::PathBuf;
use std::sync::Arc;

use ouroboros_relay::{router, AppState};

#[tokio::main]
async fn main() {
    let db_path = env_path("SQLITE_PATH", "/tmp/relay.sqlite");
    let snapshot_path = env_path("SNAPSHOT_PATH", "/tmp/relay.snapshot");
    let state = AppState::open(db_path, snapshot_path, 30)
        .await
        .expect("relay database opens");
    let app = router(Arc::new(state));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("bind 8080");
    axum::serve(listener, app).await.expect("serve");
}

fn env_path(name: &str, default: &str) -> PathBuf {
    std::env::var(name)
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(default))
}
