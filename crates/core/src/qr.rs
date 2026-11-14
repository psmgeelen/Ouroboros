use serde::{Deserialize, Serialize};

use crate::CryptoError;

/// Pairing invite carried in a QR code. The account code is not a field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QrInvite {
    /// Payload version.
    pub v: u32,
    /// Relay base URL.
    pub relay: String,
    /// Partnership identifier.
    pub partnership_id: String,
    /// Tracker signing public key, hex.
    pub tracker_public_key: String,
    /// One-time invite token.
    pub invite: String,
}

/// Decode a QR payload. An `account_code` field is rejected.
pub fn decode_invite(bytes: &[u8]) -> Result<QrInvite, CryptoError> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| CryptoError::InviteRejected)?;
    if value.get("account_code").is_some() {
        return Err(CryptoError::InviteRejected);
    }
    serde_json::from_value(value).map_err(|_| CryptoError::InviteRejected)
}

/// Short authentication string derived from the handshake transcript.
pub fn short_code(transcript: &[u8]) -> String {
    let hash = blake3::hash(transcript);
    let n = u32::from_be_bytes(hash.as_bytes()[0..4].try_into().expect("4 bytes")) % 1_000_000;
    format!("{n:06}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_code_is_stable() {
        let left = short_code(b"relay|partnership|key");
        let again = short_code(b"relay|partnership|key");
        let changed = short_code(b"relay|partnership|key2");
        assert_eq!(left, again);
        assert_ne!(left, changed);
        assert_eq!(left.len(), 6);
    }

    #[test]
    fn account_code_field_is_rejected() {
        let payload = br#"{"v":1,"relay":"https://relay.example","partnership_id":"p","tracker_public_key":"aa","invite":"once","account_code":"1234567890123456"}"#;
        assert_eq!(decode_invite(payload), Err(CryptoError::InviteRejected));
    }

    #[test]
    fn tampered_public_key_changes_short_code() {
        let base = b"https://relay.example|partnership|abcdef";
        let tampered = b"https://relay.example|partnership|abcdee";
        assert_ne!(short_code(base), short_code(tampered));
    }
}
