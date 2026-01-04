//! Proof of Work types.

use alloc::string::String;
use core::fmt;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Proof of Work value (8 bytes / u64).
///
/// Work is computed by finding a nonce such that the Blake2b hash of
/// (nonce || block_hash) meets a minimum difficulty threshold.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Work(u64);

impl Work {
    /// Zero work.
    pub const ZERO: Work = Work(0);

    /// Create from u64.
    pub const fn new(value: u64) -> Self {
        Work(value)
    }

    /// Get the inner u64 value.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    /// Convert to bytes (little-endian, as used in work computation).
    pub fn to_le_bytes(&self) -> [u8; 8] {
        self.0.to_le_bytes()
    }

    /// Create from little-endian bytes.
    pub fn from_le_bytes(bytes: [u8; 8]) -> Self {
        Work(u64::from_le_bytes(bytes))
    }

    /// Convert to big-endian bytes (for display/serialization).
    pub fn to_be_bytes(&self) -> [u8; 8] {
        self.0.to_be_bytes()
    }

    /// Create from big-endian bytes.
    pub fn from_be_bytes(bytes: [u8; 8]) -> Self {
        Work(u64::from_be_bytes(bytes))
    }

    /// Convert to hex string (16 characters, lowercase).
    /// Note: Work is displayed in big-endian format in Nano.
    pub fn to_hex(&self) -> String {
        hex::encode(self.to_be_bytes())
    }

    /// Create from hex string.
    pub fn from_hex(s: &str) -> Result<Self> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 8 {
            return Err(Error::InvalidWork);
        }
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&bytes);
        Ok(Work::from_be_bytes(arr))
    }

    /// Check if this is zero work.
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl fmt::Debug for Work {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Work({})", self.to_hex())
    }
}

impl fmt::Display for Work {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl From<u64> for Work {
    fn from(value: u64) -> Self {
        Work(value)
    }
}

impl From<Work> for u64 {
    fn from(work: Work) -> u64 {
        work.0
    }
}

impl Serialize for Work {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Work {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Work::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_WORK_HEX: &str = "7202df8a7c380578";

    #[test]
    fn test_work_from_hex() {
        let work = Work::from_hex(TEST_WORK_HEX).unwrap();
        assert_eq!(work.to_hex(), TEST_WORK_HEX);
    }

    #[test]
    fn test_work_zero() {
        let zero = Work::ZERO;
        assert!(zero.is_zero());
        assert_eq!(zero.to_hex(), "0000000000000000");
    }

    #[test]
    fn test_work_roundtrip() {
        let work = Work::new(0x123456789ABCDEF0);

        // Hex roundtrip
        let hex_str = work.to_hex();
        let recovered = Work::from_hex(&hex_str).unwrap();
        assert_eq!(work, recovered);

        // LE bytes roundtrip
        let le_bytes = work.to_le_bytes();
        let from_le = Work::from_le_bytes(le_bytes);
        assert_eq!(work, from_le);

        // BE bytes roundtrip
        let be_bytes = work.to_be_bytes();
        let from_be = Work::from_be_bytes(be_bytes);
        assert_eq!(work, from_be);
    }

    #[test]
    fn test_work_invalid_length() {
        let result = Work::from_hex("ABCD");
        assert!(matches!(result, Err(Error::InvalidWork)));
    }

    #[test]
    fn test_work_serde() {
        let work = Work::from_hex(TEST_WORK_HEX).unwrap();
        let json = serde_json::to_string(&work).unwrap();
        assert_eq!(json, format!("\"{}\"", TEST_WORK_HEX));

        let recovered: Work = serde_json::from_str(&json).unwrap();
        assert_eq!(work, recovered);
    }

    #[test]
    fn test_work_from_u64() {
        let work: Work = 12345u64.into();
        assert_eq!(work.as_u64(), 12345);

        let value: u64 = work.into();
        assert_eq!(value, 12345);
    }
}
