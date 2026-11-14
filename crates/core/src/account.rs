use hmac::{Hmac, Mac};
use rand::Rng;
use sha2::Sha256;
use subtle::ConstantTimeEq;

use crate::CryptoError;

type HmacSha256 = Hmac<Sha256>;

/// A 16-digit anonymous account code. The raw value is never written to logs by this crate.
#[derive(Clone, PartialEq, Eq)]
pub struct AccountCode(String);

impl AccountCode {
    /// Draw a new code from `rng`.
    pub fn generate<R: Rng>(rng: &mut R) -> Self {
        let mut chars = String::with_capacity(16);
        for _ in 0..16 {
            let digit = rng.gen_range(0u8..10);
            chars.push(char::from(b'0' + digit));
        }
        Self(chars)
    }

    /// Borrow the digits.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for AccountCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AccountCode([redacted])")
    }
}

/// Peppered HMAC of an account code. The raw code is not recoverable from the result.
pub fn hash_account_code(code: &str, pepper: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(pepper).expect("hmac accepts any pepper length");
    mac.update(code.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/// Return whether `code` matches a stored peppered hash.
pub fn verify_account_code(code: &str, pepper: &[u8], stored: &str) -> Result<(), CryptoError> {
    let computed = hash_account_code(code, pepper);
    let computed_bytes = computed.as_bytes();
    let stored_bytes = stored.as_bytes();
    if computed_bytes.len() == stored_bytes.len() && bool::from(computed_bytes.ct_eq(stored_bytes))
    {
        Ok(())
    } else {
        Err(CryptoError::AccountCodeRejected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;

    #[test]
    fn wrong_account_code_does_not_verify() {
        let code = AccountCode::generate(&mut OsRng);
        let pepper = b"test-pepper";
        let stored = hash_account_code(code.as_str(), pepper);
        assert!(!stored.contains(code.as_str()));
        assert_eq!(
            verify_account_code("0000000000000000", pepper, &stored),
            Err(CryptoError::AccountCodeRejected)
        );
        assert!(verify_account_code(code.as_str(), pepper, &stored).is_ok());
    }
}
