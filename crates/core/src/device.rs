use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use x25519_dalek::{PublicKey, StaticSecret};

use crate::CryptoError;

/// Device identity. The signing key proves provenance. The agreement key receives sealed epoch keys.
pub struct DeviceIdentity {
    signing: SigningKey,
    agreement: StaticSecret,
}

impl DeviceIdentity {
    /// Generate a new identity.
    pub fn generate() -> Self {
        let signing = SigningKey::generate(&mut OsRng);
        let agreement = StaticSecret::random_from_rng(OsRng);
        Self { signing, agreement }
    }

    /// Public material that can be enrolled and put in a QR code.
    pub fn public(&self) -> DevicePublic {
        DevicePublic {
            signing: self.signing.verifying_key(),
            agreement: PublicKey::from(&self.agreement),
        }
    }

    pub(crate) fn agreement(&self) -> &StaticSecret {
        &self.agreement
    }

    /// Sign `message`.
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.signing.sign(message).to_bytes().to_vec()
    }
}

/// Public device identity.
#[derive(Clone)]
pub struct DevicePublic {
    signing: VerifyingKey,
    agreement: PublicKey,
}

impl DevicePublic {
    /// Hex fingerprint of the signing key, safe to show to a partner.
    pub fn fingerprint(&self) -> String {
        let hash = blake3::hash(self.signing.as_bytes());
        hex::encode(&hash.as_bytes()[..8])
    }

    pub(crate) fn agreement(&self) -> &PublicKey {
        &self.agreement
    }

    /// Encode the signing key as hex for storage on the relay.
    pub fn signing_key_hex(&self) -> String {
        hex::encode(self.signing.as_bytes())
    }

    /// Restore a public signing key from hex.
    pub fn signing_key_from_hex(value: &str) -> Result<VerifyingKey, CryptoError> {
        let bytes = hex::decode(value).map_err(|_| CryptoError::BadSignature)?;
        let bytes: [u8; 32] = bytes.try_into().map_err(|_| CryptoError::BadSignature)?;
        VerifyingKey::from_bytes(&bytes).map_err(|_| CryptoError::BadSignature)
    }
}

/// Verify a detached signature. The relay calls this and does not open seals.
pub fn verify_detached(
    public_hex: &str,
    message: &[u8],
    signature: &[u8],
) -> Result<(), CryptoError> {
    let key = DevicePublic::signing_key_from_hex(public_hex)?;
    let sig_bytes: [u8; 64] = signature
        .try_into()
        .map_err(|_| CryptoError::BadSignature)?;
    let signature = ed25519_dalek::Signature::from_bytes(&sig_bytes);
    key.verify(message, &signature)
        .map_err(|_| CryptoError::BadSignature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bad_signature_is_rejected() {
        let author = DeviceIdentity::generate();
        let other = DeviceIdentity::generate();
        let message = b"partner-view";
        let signature = author.sign(message);
        assert!(verify_detached(&author.public().signing_key_hex(), message, &signature).is_ok());
        assert_eq!(
            verify_detached(&other.public().signing_key_hex(), message, &signature),
            Err(CryptoError::BadSignature)
        );
    }
}
