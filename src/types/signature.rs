//! Ed25519 signature type.

use alloc::string::String;
use core::fmt;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Ed25519 signature (64 bytes).
///
/// Used to sign Nano blocks, proving ownership of the account.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Signature([u8; 64]);

impl Signature {
    /// Create from raw bytes.
    pub const fn from_bytes(bytes: [u8; 64]) -> Self {
        Signature(bytes)
    }

    /// Get as raw bytes.
    pub const fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }

    /// Convert to hex string.
    pub fn to_hex(&self) -> String {
        hex::encode_upper(self.0)
    }

    /// Create from hex string.
    pub fn from_hex(s: &str) -> Result<Self> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 64 {
            return Err(Error::InvalidSignature);
        }
        let mut arr = [0u8; 64];
        arr.copy_from_slice(&bytes);
        Ok(Signature(arr))
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Signature({}...)", &self.to_hex()[..16])
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl From<[u8; 64]> for Signature {
    fn from(bytes: [u8; 64]) -> Self {
        Signature(bytes)
    }
}

impl AsRef<[u8]> for Signature {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Signature::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SIG_HEX: &str = "82D41BC16F313E4B2243D14DFFA2FB04679C540C2095FEE7EAE0F2F26880AD56DD48D87A7CC5DD760C5B2D76EE2C205506AA557BF00B60D8DEE312EC7343A501";

    #[test]
    fn test_signature_from_hex() {
        let sig = Signature::from_hex(TEST_SIG_HEX).unwrap();
        assert_eq!(sig.to_hex(), TEST_SIG_HEX);
    }

    #[test]
    fn test_signature_roundtrip() {
        let bytes = [0xABu8; 64];
        let sig = Signature::from_bytes(bytes);
        assert_eq!(sig.as_bytes(), &bytes);

        let hex_str = sig.to_hex();
        let recovered = Signature::from_hex(&hex_str).unwrap();
        assert_eq!(sig, recovered);
    }

    #[test]
    fn test_signature_invalid_length() {
        let result = Signature::from_hex("ABCD");
        assert!(matches!(result, Err(Error::InvalidSignature)));
    }

    #[test]
    fn test_signature_serde() {
        let sig = Signature::from_hex(TEST_SIG_HEX).unwrap();
        let json = serde_json::to_string(&sig).unwrap();
        assert_eq!(json, format!("\"{}\"", TEST_SIG_HEX));

        let recovered: Signature = serde_json::from_str(&json).unwrap();
        assert_eq!(sig, recovered);
    }

    #[test]
    fn test_signature_debug() {
        let sig = Signature::from_hex(TEST_SIG_HEX).unwrap();
        let debug = format!("{:?}", sig);
        assert!(debug.starts_with("Signature("));
        assert!(debug.ends_with("...)"));
    }
}
