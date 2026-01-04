//! Account and public key types for Nano.

use alloc::format;
use alloc::string::{String, ToString};
use core::fmt;
use core::str::FromStr;
use serde::{Deserialize, Serialize};

use crate::constants::{ACCOUNT_PREFIX_NANO, ACCOUNT_PREFIX_XNO, BASE32_ALPHABET};
use crate::error::{AccountError, Error, Result};

/// Public key (32 bytes).
///
/// Represents an Ed25519 public key used in the Nano network.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PublicKey([u8; 32]);

impl PublicKey {
    /// Zero public key (burn address).
    pub const ZERO: PublicKey = PublicKey([0u8; 32]);

    /// Create from raw bytes.
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        PublicKey(bytes)
    }

    /// Get as raw bytes.
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Convert to hex string.
    pub fn to_hex(&self) -> String {
        hex::encode_upper(self.0)
    }

    /// Create from hex string.
    pub fn from_hex(s: &str) -> Result<Self> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 32 {
            return Err(Error::InvalidPublicKey);
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(PublicKey(arr))
    }

    /// Convert to Account address.
    pub fn to_account(&self) -> Account {
        Account::from_public_key(self)
    }

    /// Check if this is the zero/burn key.
    pub fn is_zero(&self) -> bool {
        self.0 == [0u8; 32]
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PublicKey({})", self.to_hex())
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl From<[u8; 32]> for PublicKey {
    fn from(bytes: [u8; 32]) -> Self {
        PublicKey(bytes)
    }
}

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        PublicKey::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

/// Nano account address.
///
/// Represents a Nano address in the format `nano_` or `xrb_` followed by
/// 52 base32-encoded characters (260 bits: 256-bit public key + 4-bit padding).
/// Includes a 5-byte checksum encoded in the last 8 characters.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Account {
    /// The underlying public key.
    public_key: PublicKey,
    /// Cached address string.
    address: String,
}

impl Account {
    /// Create an account from a public key.
    pub fn from_public_key(public_key: &PublicKey) -> Self {
        let address = encode_account(public_key);
        Account {
            public_key: *public_key,
            address,
        }
    }

    /// Parse an account from an address string.
    pub fn from_address_str_checked(s: &str) -> Result<Self> {
        let public_key = decode_account(s)?;
        let address = s.to_string();
        Ok(Account {
            public_key,
            address,
        })
    }

    /// Get the underlying public key.
    pub const fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Get the address string.
    pub fn as_str(&self) -> &str {
        &self.address
    }

    /// Check if this is the burn address.
    pub fn is_burn(&self) -> bool {
        self.public_key.is_zero()
    }
}

impl fmt::Debug for Account {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Account({})", self.address)
    }
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.address)
    }
}

impl FromStr for Account {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Account::from_address_str_checked(s)
    }
}

impl From<PublicKey> for Account {
    fn from(public_key: PublicKey) -> Self {
        Account::from_public_key(&public_key)
    }
}

impl Serialize for Account {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.address)
    }
}

impl<'de> Deserialize<'de> for Account {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Account::from_address_str_checked(&s).map_err(serde::de::Error::custom)
    }
}

/// Encode a public key to a Nano account address.
fn encode_account(public_key: &PublicKey) -> String {
    use blake2::digest::consts::U5;
    use blake2::{Blake2b, Digest};

    // Calculate checksum (5 bytes)
    let mut hasher = Blake2b::<U5>::new();
    hasher.update(public_key.as_bytes());
    let mut checksum: [u8; 5] = hasher.finalize().into();

    checksum.reverse();

    // Encode public key (256 bits) with 4-bit padding = 260 bits = 52 base32 chars
    let pk_encoded = encode_base32_256(public_key.as_bytes());

    // Encode checksum (40 bits) = 8 base32 chars
    let checksum_encoded = encode_base32_40(&checksum);

    format!("{}{}{}", ACCOUNT_PREFIX_NANO, pk_encoded, checksum_encoded)
}

/// Decode a Nano account address to a public key.
fn decode_account(address: &str) -> Result<PublicKey> {
    use blake2::digest::consts::U5;
    use blake2::{Blake2b, Digest};

    // Check prefix
    let data = if let Some(s) = address.strip_prefix(ACCOUNT_PREFIX_NANO) {
        s
    } else if let Some(s) = address.strip_prefix(ACCOUNT_PREFIX_XNO) {
        s
    } else {
        return Err(Error::InvalidAccount(AccountError::InvalidPrefix));
    };

    // Check length: 52 chars for public key + 8 chars for checksum = 60 chars
    if data.len() != 60 {
        return Err(Error::InvalidAccount(AccountError::InvalidLength));
    }

    let pk_part = &data[..52];
    let checksum_part = &data[52..];

    // Decode public key
    let public_key_bytes = decode_base32_256(pk_part)
        .map_err(|_| Error::InvalidAccount(AccountError::InvalidEncoding))?;

    // Decode checksum
    let mut checksum_bytes = decode_base32_40(checksum_part)
        .map_err(|_| Error::InvalidAccount(AccountError::InvalidEncoding))?;

    checksum_bytes.reverse();

    // Calculate expected checksum
    let mut hasher = Blake2b::<U5>::new();
    hasher.update(&public_key_bytes);
    let expected_checksum: [u8; 5] = hasher.finalize().into();

    if checksum_bytes != expected_checksum {
        return Err(Error::InvalidAccount(AccountError::ChecksumMismatch));
    }

    Ok(PublicKey::from_bytes(public_key_bytes))
}

/// Encode 256 bits (32 bytes) to 52 base32 characters.
fn encode_base32_256(bytes: &[u8; 32]) -> String {
    // 256 bits + 4 bits padding = 260 bits = 52 * 5 bits
    let mut result = String::with_capacity(52);

    // Process 256 bits in groups of 5 bits
    // We'll use a bit accumulator approach

    // First character has 4 bits of padding (zeros) + 1 bit from first byte
    let mut bits = (bytes[0] >> 7) as u16;
    result.push(BASE32_ALPHABET[bits as usize] as char);

    // Remaining processing
    bits = ((bytes[0] >> 2) & 0x1F) as u16;
    result.push(BASE32_ALPHABET[bits as usize] as char);

    bits = (bytes[0] & 0x03) as u16;
    let mut bit_count: u8 = 2;

    for &byte in &bytes[1..] {
        bits = (bits << 8) | (byte as u16);
        bit_count += 8;

        while bit_count >= 5 {
            bit_count -= 5;
            let idx = ((bits >> bit_count) & 0x1F) as usize;
            result.push(BASE32_ALPHABET[idx] as char);
        }
        bits &= (1 << bit_count) - 1;
    }

    if bit_count > 0 {
        bits <<= 5 - bit_count;
        result.push(BASE32_ALPHABET[(bits & 0x1F) as usize] as char);
    }

    result
}

/// Decode 52 base32 characters to 256 bits (32 bytes).
fn decode_base32_256(s: &str) -> core::result::Result<[u8; 32], ()> {
    if s.len() != 52 {
        return Err(());
    }

    let mut result = [0u8; 32];
    let mut bits: u32 = 0;
    let mut bit_count: u8 = 0;
    let mut byte_idx = 0;

    for (i, c) in s.chars().enumerate() {
        let value = base32_char_value(c)?;

        if i == 0 {
            // First char has 4 bits padding, only use lowest bit
            bits = (value & 0x01) as u32;
            bit_count = 1;
        } else {
            bits = (bits << 5) | (value as u32);
            bit_count += 5;
        }

        while bit_count >= 8 && byte_idx < 32 {
            bit_count -= 8;
            result[byte_idx] = ((bits >> bit_count) & 0xFF) as u8;
            byte_idx += 1;
        }
        bits &= (1 << bit_count) - 1;
    }

    if byte_idx != 32 {
        return Err(());
    }

    Ok(result)
}

/// Encode 40 bits (5 bytes) to 8 base32 characters.
fn encode_base32_40(bytes: &[u8; 5]) -> String {
    let mut result = String::with_capacity(8);

    // 40 bits = 8 * 5 bits
    let combined: u64 = ((bytes[0] as u64) << 32)
        | ((bytes[1] as u64) << 24)
        | ((bytes[2] as u64) << 16)
        | ((bytes[3] as u64) << 8)
        | (bytes[4] as u64);

    for i in (0..8).rev() {
        let idx = ((combined >> (i * 5)) & 0x1F) as usize;
        result.push(BASE32_ALPHABET[idx] as char);
    }

    result
}

/// Decode 8 base32 characters to 40 bits (5 bytes).
fn decode_base32_40(s: &str) -> core::result::Result<[u8; 5], ()> {
    if s.len() != 8 {
        return Err(());
    }

    let mut combined: u64 = 0;

    for c in s.chars() {
        let value = base32_char_value(c)?;
        combined = (combined << 5) | (value as u64);
    }

    Ok([
        ((combined >> 32) & 0xFF) as u8,
        ((combined >> 24) & 0xFF) as u8,
        ((combined >> 16) & 0xFF) as u8,
        ((combined >> 8) & 0xFF) as u8,
        (combined & 0xFF) as u8,
    ])
}

/// Get the value of a base32 character.
fn base32_char_value(c: char) -> core::result::Result<u8, ()> {
    match c {
        '1' => Ok(0),
        '3' => Ok(1),
        '4' => Ok(2),
        '5' => Ok(3),
        '6' => Ok(4),
        '7' => Ok(5),
        '8' => Ok(6),
        '9' => Ok(7),
        'a' | 'A' => Ok(8),
        'b' | 'B' => Ok(9),
        'c' | 'C' => Ok(10),
        'd' | 'D' => Ok(11),
        'e' | 'E' => Ok(12),
        'f' | 'F' => Ok(13),
        'g' | 'G' => Ok(14),
        'h' | 'H' => Ok(15),
        'i' | 'I' => Ok(16),
        'j' | 'J' => Ok(17),
        'k' | 'K' => Ok(18),
        'm' | 'M' => Ok(19),
        'n' | 'N' => Ok(20),
        'o' | 'O' => Ok(21),
        'p' | 'P' => Ok(22),
        'q' | 'Q' => Ok(23),
        'r' | 'R' => Ok(24),
        's' | 'S' => Ok(25),
        't' | 'T' => Ok(26),
        'u' | 'U' => Ok(27),
        'w' | 'W' => Ok(28),
        'x' | 'X' => Ok(29),
        'y' | 'Y' => Ok(30),
        'z' | 'Z' => Ok(31),
        _ => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test vector from Nano documentation
    const TEST_PUBLIC_KEY_HEX: &str =
        "E89208DD038FBB269987689621D52292AE9C35941A7484756ECCED92A65093BA";
    const TEST_ACCOUNT: &str = "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3";

    #[test]
    fn test_public_key_from_hex() {
        let pk = PublicKey::from_hex(TEST_PUBLIC_KEY_HEX).unwrap();
        assert_eq!(pk.to_hex(), TEST_PUBLIC_KEY_HEX);
    }

    #[test]
    fn test_public_key_to_account() {
        let pk = PublicKey::from_hex(TEST_PUBLIC_KEY_HEX).unwrap();
        let account = pk.to_account();
        assert_eq!(account.as_str(), TEST_ACCOUNT);
    }

    #[test]
    fn test_account_from_str() {
        let account: Account = TEST_ACCOUNT.parse().unwrap();
        assert_eq!(account.public_key().to_hex(), TEST_PUBLIC_KEY_HEX);
    }

    #[test]
    fn test_account_xno_prefix() {
        let xno_account = TEST_ACCOUNT.replace("nano_", "xno_");
        let account: Account = xno_account.parse().unwrap();
        assert_eq!(account.public_key().to_hex(), TEST_PUBLIC_KEY_HEX);
    }

    #[test]
    fn test_account_roundtrip() {
        let pk = PublicKey::from_hex(TEST_PUBLIC_KEY_HEX).unwrap();
        let account = Account::from_public_key(&pk);
        let parsed: Account = account.as_str().parse().unwrap();
        assert_eq!(parsed.public_key(), &pk);
    }

    #[test]
    fn test_invalid_account_prefix() {
        let invalid = "invalid_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3";
        assert!(matches!(
            Account::from_address_str_checked(invalid),
            Err(Error::InvalidAccount(AccountError::InvalidPrefix))
        ));
    }

    #[test]
    fn test_invalid_account_length() {
        let invalid = "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuo";
        assert!(matches!(
            Account::from_address_str_checked(invalid),
            Err(Error::InvalidAccount(AccountError::InvalidLength))
        ));
    }

    #[test]
    fn test_invalid_account_checksum() {
        let invalid = "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr4";
        assert!(matches!(
            Account::from_address_str_checked(invalid),
            Err(Error::InvalidAccount(AccountError::ChecksumMismatch))
        ));
    }

    #[test]
    fn test_public_key_zero() {
        let zero = PublicKey::ZERO;
        assert!(zero.is_zero());
        assert!(!PublicKey::from_hex(TEST_PUBLIC_KEY_HEX).unwrap().is_zero());
    }

    #[test]
    fn test_burn_address() {
        let zero = PublicKey::ZERO;
        let account = zero.to_account();
        assert!(account.is_burn());
    }

    #[test]
    fn test_public_key_serde() {
        let pk = PublicKey::from_hex(TEST_PUBLIC_KEY_HEX).unwrap();
        let json = serde_json::to_string(&pk).unwrap();
        assert_eq!(json, format!("\"{}\"", TEST_PUBLIC_KEY_HEX));

        let recovered: PublicKey = serde_json::from_str(&json).unwrap();
        assert_eq!(pk, recovered);
    }

    #[test]
    fn test_account_serde() {
        let account: Account = TEST_ACCOUNT.parse().unwrap();
        let json = serde_json::to_string(&account).unwrap();
        assert_eq!(json, format!("\"{}\"", TEST_ACCOUNT));

        let recovered: Account = serde_json::from_str(&json).unwrap();
        assert_eq!(account, recovered);
    }

    #[test]
    fn test_base32_roundtrip() {
        let bytes = [0xABu8; 32];
        let encoded = encode_base32_256(&bytes);
        let decoded = decode_base32_256(&encoded).unwrap();
        assert_eq!(bytes, decoded);
    }

    #[test]
    fn test_base32_checksum_roundtrip() {
        let bytes = [0xCDu8; 5];
        let encoded = encode_base32_40(&bytes);
        let decoded = decode_base32_40(&encoded).unwrap();
        assert_eq!(bytes, decoded);
    }

    #[test]
    fn test_multiple_accounts() {
        let test_cases = [
            (
                "0000000000000000000000000000000000000000000000000000000000000000",
                "nano_1111111111111111111111111111111111111111111111111111hifc8npp",
            ),
            (
                "E89208DD038FBB269987689621D52292AE9C35941A7484756ECCED92A65093BA",
                "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3",
            ),
        ];

        for (pk_hex, expected_account) in test_cases {
            let pk = PublicKey::from_hex(pk_hex).unwrap();
            let account = pk.to_account();
            assert_eq!(account.as_str(), expected_account);

            let parsed: Account = expected_account.parse().unwrap();
            assert_eq!(parsed.public_key().to_hex(), pk_hex);
        }
    }
}
