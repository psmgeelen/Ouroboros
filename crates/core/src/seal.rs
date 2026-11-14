use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use rand::rngs::OsRng;
use rand::RngCore;
use x25519_dalek::{EphemeralSecret, PublicKey};

use crate::device::{DeviceIdentity, DevicePublic};
use crate::CryptoError;

/// An epoch key sealed to a partner agreement key. Opening it requires the partner secret.
#[derive(Clone, PartialEq, Eq)]
pub struct SealedEpochKey {
    /// Ephemeral X25519 public key.
    pub ephemeral_public: [u8; 32],
    /// XChaCha20-Poly1305 nonce.
    pub nonce: [u8; 24],
    /// Encrypted 32-byte epoch key.
    pub ciphertext: Vec<u8>,
}

impl SealedEpochKey {
    /// Seal `epoch_key` so only `partner` can open it.
    pub fn seal(epoch_key: &[u8; 32], partner: &DevicePublic) -> Self {
        let ephemeral = EphemeralSecret::random_from_rng(OsRng);
        let ephemeral_public = PublicKey::from(&ephemeral);
        let shared = ephemeral.diffie_hellman(partner.agreement());
        let key = derive_key(
            shared.as_bytes(),
            ephemeral_public.as_bytes(),
            partner.agreement().as_bytes(),
        );
        let mut nonce = [0u8; 24];
        OsRng.fill_bytes(&mut nonce);
        let cipher = XChaCha20Poly1305::new((&key).into());
        let ciphertext = cipher
            .encrypt(
                XNonce::from_slice(&nonce),
                Payload {
                    msg: epoch_key,
                    aad: ephemeral_public.as_bytes(),
                },
            )
            .expect("encrypting a 32-byte key cannot fail");
        Self {
            ephemeral_public: *ephemeral_public.as_bytes(),
            nonce,
            ciphertext,
        }
    }

    /// Open a seal with the partner identity. Returns the epoch key.
    pub fn open(&self, partner: &DeviceIdentity) -> Result<[u8; 32], CryptoError> {
        let ephemeral_public = PublicKey::from(self.ephemeral_public);
        let shared = partner.agreement().diffie_hellman(&ephemeral_public);
        let partner_public = PublicKey::from(partner.agreement());
        let key = derive_key(
            shared.as_bytes(),
            ephemeral_public.as_bytes(),
            partner_public.as_bytes(),
        );
        let cipher = XChaCha20Poly1305::new((&key).into());
        let plain = cipher
            .decrypt(
                XNonce::from_slice(&self.nonce),
                Payload {
                    msg: self.ciphertext.as_ref(),
                    aad: ephemeral_public.as_bytes(),
                },
            )
            .map_err(|_| CryptoError::SealRejected)?;
        let key: [u8; 32] = plain.try_into().map_err(|_| CryptoError::SealRejected)?;
        Ok(key)
    }
}

fn derive_key(shared: &[u8], ephemeral_public: &[u8], partner_public: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new_derive_key("ouroboros epoch seal v1");
    hasher.update(shared);
    hasher.update(ephemeral_public);
    hasher.update(partner_public);
    *hasher.finalize().as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngCore;

    #[test]
    fn deleted_seal_does_not_decrypt() {
        let partner = DeviceIdentity::generate();
        let mut epoch = [0u8; 32];
        OsRng.fill_bytes(&mut epoch);
        let mut seal = SealedEpochKey::seal(&epoch, &partner.public());
        let ciphertext = seal.ciphertext.clone();
        seal.ciphertext.clear();
        assert!(!ciphertext.is_empty());
        assert_eq!(seal.open(&partner), Err(CryptoError::SealRejected));
    }
}
