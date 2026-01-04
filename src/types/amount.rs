//! Amount types for representing Nano values.

use alloc::format;
use alloc::string::{String, ToString};
use core::fmt;
use core::ops::{Add, Sub};
use core::str::FromStr;
use serde::{Deserialize, Serialize};

use crate::constants::NANO_IN_RAW;
use crate::error::{AmountError, Error, Result};

/// Raw amount - the smallest unit of Nano (10^-30 XNO).
///
/// This is a newtype wrapper around u128 representing raw units.
/// All internal calculations are done in raw to avoid floating point errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Raw(u128);

impl Raw {
    /// Zero raw amount.
    pub const ZERO: Raw = Raw(0);

    /// Maximum possible raw amount.
    pub const MAX: Raw = Raw(u128::MAX);

    /// Create a new raw amount from u128.
    #[inline]
    pub const fn new(value: u128) -> Self {
        Raw(value)
    }

    /// Get the inner u128 value.
    #[inline]
    pub const fn as_u128(&self) -> u128 {
        self.0
    }

    /// Create from Nano (XNO) units (1 XNO = 10^30 raw).
    pub fn from_nano(nano: u128) -> Result<Self> {
        nano.checked_mul(NANO_IN_RAW)
            .map(Raw)
            .ok_or(Error::InvalidAmount(AmountError::Overflow))
    }

    /// Convert to Nano (XNO) as a string with decimal places.
    pub fn to_nano_string(&self) -> String {
        let whole = self.0 / NANO_IN_RAW;
        let frac = self.0 % NANO_IN_RAW;

        if frac == 0 {
            whole.to_string()
        } else {
            let frac_str = format!("{:030}", frac);
            let trimmed = frac_str.trim_end_matches('0');
            format!("{}.{}", whole, trimmed)
        }
    }

    /// Check if the amount is zero.
    #[inline]
    pub const fn is_zero(&self) -> bool {
        self.0 == 0
    }

    /// Checked addition.
    pub fn checked_add(&self, other: Raw) -> Option<Raw> {
        self.0.checked_add(other.0).map(Raw)
    }

    /// Checked subtraction.
    pub fn checked_sub(&self, other: Raw) -> Option<Raw> {
        self.0.checked_sub(other.0).map(Raw)
    }

    /// Saturating addition.
    pub fn saturating_add(&self, other: Raw) -> Raw {
        Raw(self.0.saturating_add(other.0))
    }

    /// Saturating subtraction.
    pub fn saturating_sub(&self, other: Raw) -> Raw {
        Raw(self.0.saturating_sub(other.0))
    }

    /// Convert to big-endian bytes (16 bytes).
    pub fn to_be_bytes(&self) -> [u8; 16] {
        self.0.to_be_bytes()
    }

    /// Create from big-endian bytes.
    pub fn from_be_bytes(bytes: [u8; 16]) -> Self {
        Raw(u128::from_be_bytes(bytes))
    }

    /// Convert to hex string (32 characters, uppercase).
    pub fn to_hex(&self) -> String {
        hex::encode_upper(self.to_be_bytes())
    }

    /// Create from hex string.
    pub fn from_hex(s: &str) -> Result<Self> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 16 {
            return Err(Error::InvalidAmount(AmountError::InvalidFormat));
        }
        let mut arr = [0u8; 16];
        arr.copy_from_slice(&bytes);
        Ok(Raw::from_be_bytes(arr))
    }
}

impl Add for Raw {
    type Output = Raw;

    fn add(self, other: Raw) -> Raw {
        Raw(self.0 + other.0)
    }
}

impl Sub for Raw {
    type Output = Raw;

    fn sub(self, other: Raw) -> Raw {
        Raw(self.0 - other.0)
    }
}

impl From<u128> for Raw {
    fn from(value: u128) -> Self {
        Raw(value)
    }
}

impl From<Raw> for u128 {
    fn from(raw: Raw) -> u128 {
        raw.0
    }
}

impl fmt::Display for Raw {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Raw {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        s.parse::<u128>()
            .map(Raw)
            .map_err(|_| Error::InvalidAmount(AmountError::InvalidFormat))
    }
}

impl Serialize for Raw {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for Raw {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// Amount with unit information for display purposes.
///
/// This is a wrapper around Raw that also stores the preferred display unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Amount {
    raw: Raw,
}

impl Amount {
    /// Create a new amount from raw units.
    pub const fn from_raw(raw: Raw) -> Self {
        Amount { raw }
    }

    /// Create a zero amount.
    pub const fn zero() -> Self {
        Amount { raw: Raw::ZERO }
    }

    /// Get the raw value.
    pub const fn raw(&self) -> Raw {
        self.raw
    }

    /// Check if the amount is zero.
    pub const fn is_zero(&self) -> bool {
        self.raw.is_zero()
    }

    /// Get as Nano (XNO) string with decimal places.
    pub fn as_nano(&self) -> String {
        self.raw.to_nano_string()
    }
}

impl From<Raw> for Amount {
    fn from(raw: Raw) -> Self {
        Amount { raw }
    }
}

impl From<u128> for Amount {
    fn from(value: u128) -> Self {
        Amount { raw: Raw(value) }
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} raw", self.raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_basic_operations() {
        let a = Raw::new(100);
        let b = Raw::new(50);

        assert_eq!(a + b, Raw::new(150));
        assert_eq!(a - b, Raw::new(50));
        assert_eq!(a.checked_add(b), Some(Raw::new(150)));
        assert_eq!(b.checked_sub(a), None);
    }

    #[test]
    fn test_raw_from_nano() {
        let raw = Raw::from_nano(1).unwrap();
        assert_eq!(raw.as_u128(), NANO_IN_RAW);
    }

    #[test]
    fn test_raw_to_nano_string() {
        let raw = Raw::new(NANO_IN_RAW);
        assert_eq!(raw.to_nano_string(), "1");

        let raw = Raw::new(NANO_IN_RAW + NANO_IN_RAW / 2);
        assert_eq!(raw.to_nano_string(), "1.5");

        let raw = Raw::new(0);
        assert_eq!(raw.to_nano_string(), "0");
    }

    #[test]
    fn test_raw_hex() {
        let raw = Raw::new(12345678901234567890);
        let hex_str = raw.to_hex();
        let recovered = Raw::from_hex(&hex_str).unwrap();
        assert_eq!(raw, recovered);
    }

    #[test]
    fn test_raw_be_bytes() {
        let raw = Raw::new(0x123456789ABCDEF0);
        let bytes = raw.to_be_bytes();
        let recovered = Raw::from_be_bytes(bytes);
        assert_eq!(raw, recovered);
    }

    #[test]
    fn test_raw_parse() {
        let raw: Raw = "1000000000000000000000000000000".parse().unwrap();
        assert_eq!(raw, Raw::from_nano(1).unwrap());
    }

    #[test]
    fn test_raw_display() {
        let raw = Raw::new(12345);
        assert_eq!(raw.to_string(), "12345");
    }

    #[test]
    fn test_raw_zero() {
        assert!(Raw::ZERO.is_zero());
        assert!(!Raw::new(1).is_zero());
    }

    #[test]
    fn test_amount_creation() {
        let amount = Amount::from_raw(Raw::new(1000));
        assert_eq!(amount.raw(), Raw::new(1000));
        assert!(!amount.is_zero());
    }

    #[test]
    fn test_amount_zero() {
        let amount = Amount::zero();
        assert!(amount.is_zero());
    }

    #[test]
    fn test_raw_overflow() {
        assert!(Raw::from_nano(u128::MAX).is_err());
    }

    #[test]
    fn test_raw_serde() {
        let raw = Raw::new(12345678901234567890);
        let json = serde_json::to_string(&raw).unwrap();
        assert_eq!(json, "\"12345678901234567890\"");

        let recovered: Raw = serde_json::from_str(&json).unwrap();
        assert_eq!(raw, recovered);
    }
}
