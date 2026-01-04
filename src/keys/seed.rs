//! Seed generation and management.

use alloc::string::String;
use core::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::{Error, Result};
use crate::keys::{derive_keypair, KeyPair};

/// Nano wallet seed (32 bytes).
///
/// The seed is the master secret from which all account keys are derived.
/// It should be kept secret and backed up securely.
///
/// Seeds are automatically zeroed when dropped for security.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Seed([u8; 32]);

impl Seed {
    /// Create a new random seed.
    ///
    /// Uses the system's cryptographically secure random number generator.
    #[cfg(any(feature = "std", feature = "wasm-rpc", feature = "wasm-websocket"))]
    pub fn random() -> Result<Self> {
        let mut bytes = [0u8; 32];
        getrandom::getrandom(&mut bytes).map_err(|_| Error::InvalidSeed)?;
        Ok(Seed(bytes))
    }

    /// Create from raw bytes.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Seed(bytes)
    }

    /// Get as raw bytes.
    ///
    /// Note: Handle with care - this exposes the secret seed.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Create from hex string.
    pub fn from_hex(s: &str) -> Result<Self> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 32 {
            return Err(Error::InvalidSeed);
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Seed(arr))
    }

    /// Convert to hex string.
    ///
    /// Note: Handle with care - this exposes the secret seed.
    pub fn to_hex(&self) -> String {
        hex::encode_upper(self.0)
    }

    /// Derive a keypair at the given index.
    ///
    /// Index 0 is the first account, index 1 is the second, etc.
    pub fn derive(&self, index: u32) -> KeyPair {
        derive_keypair(&self.0, index)
    }
}

impl fmt::Debug for Seed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Seed([REDACTED])")
    }
}

impl PartialEq for Seed {
    fn eq(&self, other: &Self) -> bool {
        // Constant-time comparison
        use subtle::ConstantTimeEq;
        self.0.ct_eq(&other.0).into()
    }
}

impl Eq for Seed {}

#[cfg(test)]
mod tests {
    use super::*;

    // Test vector from Nano documentation
    const TEST_SEED_HEX: &str = "0000000000000000000000000000000000000000000000000000000000000000";

    #[test]
    fn test_seed_from_hex() {
        let seed = Seed::from_hex(TEST_SEED_HEX).unwrap();
        assert_eq!(seed.to_hex(), TEST_SEED_HEX);
    }

    #[test]
    fn test_seed_from_bytes() {
        let bytes = [0u8; 32];
        let seed = Seed::from_bytes(bytes);
        assert_eq!(seed.as_bytes(), &bytes);
    }

    #[test]
    fn test_seed_invalid_hex_length() {
        let result = Seed::from_hex("ABCD");
        assert!(matches!(result, Err(Error::InvalidSeed)));
    }

    #[test]
    fn test_seed_debug_redacted() {
        let seed = Seed::from_hex(TEST_SEED_HEX).unwrap();
        let debug = format!("{:?}", seed);
        assert_eq!(debug, "Seed([REDACTED])");
        assert!(!debug.contains("0000"));
    }

    #[test]
    fn test_seed_derive() {
        let seed = Seed::from_hex(TEST_SEED_HEX).unwrap();
        let keypair0 = seed.derive(0);
        let keypair1 = seed.derive(1);

        // Different indices should produce different keys
        assert_ne!(keypair0.public_key(), keypair1.public_key());

        // Same index should produce same keys
        let keypair0_again = seed.derive(0);
        assert_eq!(keypair0.public_key(), keypair0_again.public_key());
    }

    #[cfg(any(feature = "std", feature = "wasm-rpc", feature = "wasm-websocket"))]
    #[test]
    fn test_seed_random() {
        let seed1 = Seed::random().unwrap();
        let seed2 = Seed::random().unwrap();

        // Random seeds should be different
        assert_ne!(seed1, seed2);
    }

    #[test]
    fn test_seed_equality() {
        let seed1 = Seed::from_hex(TEST_SEED_HEX).unwrap();
        let seed2 = Seed::from_hex(TEST_SEED_HEX).unwrap();
        let seed3 =
            Seed::from_hex("1111111111111111111111111111111111111111111111111111111111111111")
                .unwrap();

        assert_eq!(seed1, seed2);
        assert_ne!(seed1, seed3);
    }
}
