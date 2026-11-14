use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::device::{verify_detached, DeviceIdentity, DevicePublic};
use crate::seal::SealedEpochKey;
use crate::CryptoError;

/// A phase window shown to a partner. It is not a log entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhaseWindow {
    /// Short phase name, such as "period".
    pub label: String,
    /// Inclusive start date, `YYYY-MM-DD`.
    pub start: String,
    /// Inclusive end date, `YYYY-MM-DD`.
    pub end: String,
}

/// The only cycle information a partner is allowed to hold.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartnerView {
    /// Unix seconds when the tracker built this view.
    pub generated_at: i64,
    /// Epoch number. A new publish uses a new epoch.
    pub epoch: u64,
    /// Phase the tracker is in now.
    pub current_phase: PhaseWindow,
    /// At most two upcoming phases.
    pub upcoming: Vec<PhaseWindow>,
    /// Confidence from 0 to 1.
    pub confidence: f64,
    /// Fingerprint of the tracker signing key.
    pub tracker_fingerprint: String,
}

impl PartnerView {
    /// Build a view. More than two upcoming phases is rejected.
    pub fn new(
        generated_at: i64,
        epoch: u64,
        current_phase: PhaseWindow,
        upcoming: Vec<PhaseWindow>,
        confidence: f64,
        tracker_fingerprint: String,
    ) -> Result<Self, CryptoError> {
        if upcoming.len() > 2 {
            return Err(CryptoError::ViewSchemaRejected);
        }
        Ok(Self {
            generated_at,
            epoch,
            current_phase,
            upcoming,
            confidence,
            tracker_fingerprint,
        })
    }
}

/// Ciphertext, sealed epoch key, and tracker signature.
#[derive(Clone)]
pub struct SignedView {
    /// Encrypted [`PartnerView`].
    pub ciphertext: Vec<u8>,
    /// Epoch key sealed to the partner.
    pub seal: SealedEpochKey,
    /// Signature over `ciphertext` by the tracker.
    pub signature: Vec<u8>,
}

/// Encrypt a partner view under `epoch_key`. This does not return cycle history.
pub fn encrypt_view(view: &PartnerView, epoch_key: &[u8; 32]) -> Result<Vec<u8>, CryptoError> {
    let plain = serde_json::to_vec(view).map_err(|_| CryptoError::ViewRejected)?;
    let mut nonce = [0u8; 24];
    OsRng.fill_bytes(&mut nonce);
    let cipher = XChaCha20Poly1305::new(epoch_key.into());
    let body = cipher
        .encrypt(
            XNonce::from_slice(&nonce),
            Payload {
                msg: &plain,
                aad: b"ouroboros-view",
            },
        )
        .map_err(|_| CryptoError::ViewRejected)?;
    let mut out = nonce.to_vec();
    out.extend(body);
    Ok(out)
}

/// Decrypt a partner view. The result has no history, notes, or account code.
pub fn decrypt_view(bytes: &[u8], epoch_key: &[u8; 32]) -> Result<PartnerView, CryptoError> {
    if bytes.len() < 24 {
        return Err(CryptoError::ViewRejected);
    }
    let (nonce, body) = bytes.split_at(24);
    let cipher = XChaCha20Poly1305::new(epoch_key.into());
    let plain = cipher
        .decrypt(
            XNonce::from_slice(nonce),
            Payload {
                msg: body,
                aad: b"ouroboros-view",
            },
        )
        .map_err(|_| CryptoError::ViewRejected)?;
    decode_partner_view(&plain)
}

/// Decode JSON and reject history, notes, logs, or an account code.
pub fn decode_partner_view(bytes: &[u8]) -> Result<PartnerView, CryptoError> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| CryptoError::ViewSchemaRejected)?;
    if let Some(object) = value.as_object() {
        for forbidden in ["history", "notes", "account_code", "logs"] {
            if object.contains_key(forbidden) {
                return Err(CryptoError::ViewSchemaRejected);
            }
        }
    }
    let view: PartnerView =
        serde_json::from_value(value).map_err(|_| CryptoError::ViewSchemaRejected)?;
    if view.upcoming.len() > 2 {
        return Err(CryptoError::ViewSchemaRejected);
    }
    Ok(view)
}

/// Encrypt, seal, and sign a partner view.
pub fn sign_view(
    tracker: &DeviceIdentity,
    partner: &DevicePublic,
    view: &PartnerView,
) -> Result<SignedView, CryptoError> {
    let mut epoch_key = [0u8; 32];
    OsRng.fill_bytes(&mut epoch_key);
    let ciphertext = encrypt_view(view, &epoch_key)?;
    let seal = SealedEpochKey::seal(&epoch_key, partner);
    let signature = tracker.sign(&ciphertext);
    Ok(SignedView {
        ciphertext,
        seal,
        signature,
    })
}

/// Verify, open the seal, and decrypt. Fails closed on a bad signature or a deleted seal.
pub fn open_partner_view(
    tracker: &DevicePublic,
    partner: &DeviceIdentity,
    signed: &SignedView,
) -> Result<PartnerView, CryptoError> {
    verify_detached(
        &tracker.signing_key_hex(),
        &signed.ciphertext,
        &signed.signature,
    )?;
    let epoch_key = signed.seal.open(partner)?;
    decrypt_view(&signed.ciphertext, &epoch_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(epoch: u64, fingerprint: &str) -> PartnerView {
        PartnerView::new(
            1,
            epoch,
            PhaseWindow {
                label: "period".into(),
                start: "2026-01-01".into(),
                end: "2026-01-05".into(),
            },
            vec![],
            0.5,
            fingerprint.into(),
        )
        .unwrap()
    }

    #[test]
    fn epoch_rotation_isolates_blobs() {
        let tracker = DeviceIdentity::generate();
        let partner = DeviceIdentity::generate();
        let first = sign_view(
            &tracker,
            &partner.public(),
            &sample(1, &tracker.public().fingerprint()),
        )
        .unwrap();
        let second = sign_view(
            &tracker,
            &partner.public(),
            &sample(2, &tracker.public().fingerprint()),
        )
        .unwrap();
        let old_key = first.seal.open(&partner).unwrap();
        let new_key = second.seal.open(&partner).unwrap();
        assert!(decrypt_view(&second.ciphertext, &old_key).is_err());
        assert!(decrypt_view(&first.ciphertext, &new_key).is_err());
        assert!(decrypt_view(&first.ciphertext, &old_key).is_ok());
    }

    #[test]
    fn history_payload_is_rejected() {
        let payload = br#"{"generated_at":1,"epoch":1,"current_phase":{"label":"period","start":"2026-01-01","end":"2026-01-05"},"upcoming":[],"confidence":0.5,"tracker_fingerprint":"abcd","history":[]}"#;
        assert_eq!(
            decode_partner_view(payload),
            Err(CryptoError::ViewSchemaRejected)
        );
    }
}
