//! Random one-time codes and session tokens, and the hashes stored in their place.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Prefix of every session token, so a leaked token is easy to recognise in logs and scanners.
pub const TOKEN_PREFIX: &str = "rcs_";

/// A six-digit code, with leading zeros kept.
pub fn new_code() -> String {
    format!("{:06}", rand::random_range(0..1_000_000u32))
}

/// The stored hash of a code. The code's ID is mixed in, so equal codes hash differently.
pub fn code_hash(code_id: Uuid, code: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(code_id.as_bytes());
    hasher.update(code.as_bytes());
    hasher.finalize().to_vec()
}

/// A new session token: the prefix and 32 random bytes, base64url-encoded.
pub fn new_token() -> String {
    let bytes: [u8; 32] = rand::random();
    format!("{TOKEN_PREFIX}{}", URL_SAFE_NO_PAD.encode(bytes))
}

/// The stored hash of a session token.
pub fn token_hash(token: &str) -> Vec<u8> {
    Sha256::digest(token.as_bytes()).to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_six_digits() {
        for _ in 0..1000 {
            let code = new_code();
            assert_eq!(code.len(), 6);
            assert!(code.bytes().all(|b| b.is_ascii_digit()));
        }
    }

    #[test]
    fn tokens_are_prefixed_unique_and_long() {
        let (a, b) = (new_token(), new_token());
        assert!(a.starts_with(TOKEN_PREFIX));
        assert_eq!(a.len(), TOKEN_PREFIX.len() + 43);
        assert_ne!(a, b);
    }

    #[test]
    fn code_hash_depends_on_the_code_id() {
        let (x, y) = (Uuid::new_v4(), Uuid::new_v4());
        assert_ne!(code_hash(x, "123456"), code_hash(y, "123456"));
        assert_eq!(code_hash(x, "123456"), code_hash(x, "123456"));
    }
}
