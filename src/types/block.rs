//! Block types for Nano state blocks.

use alloc::string::{String, ToString};
use core::fmt;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::types::{Account, PublicKey, Raw, Signature, Work};

/// Block hash (32 bytes).
///
/// Represents the Blake2b-256 hash of a block's contents.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockHash([u8; 32]);

impl BlockHash {
    /// Zero hash (used for open blocks).
    pub const ZERO: BlockHash = BlockHash([0u8; 32]);

    /// Create from raw bytes.
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        BlockHash(bytes)
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
            return Err(Error::InvalidBlockHash);
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(BlockHash(arr))
    }

    /// Check if this is the zero hash.
    pub fn is_zero(&self) -> bool {
        self.0 == [0u8; 32]
    }
}

impl fmt::Debug for BlockHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockHash({})", self.to_hex())
    }
}

impl fmt::Display for BlockHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl From<[u8; 32]> for BlockHash {
    fn from(bytes: [u8; 32]) -> Self {
        BlockHash(bytes)
    }
}

impl AsRef<[u8]> for BlockHash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Serialize for BlockHash {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for BlockHash {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        BlockHash::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

/// Link field in a state block.
///
/// The link field has different meanings depending on block subtype:
/// - Send: Destination account's public key
/// - Receive/Open: Source block hash
/// - Change: Zero (unused)
/// - Epoch: Epoch signer's public key
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Link([u8; 32]);

impl Link {
    /// Zero link (used for change blocks).
    pub const ZERO: Link = Link([0u8; 32]);

    /// Create from raw bytes.
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Link(bytes)
    }

    /// Get as raw bytes.
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Create from a destination account (for send blocks).
    pub fn from_account(account: &Account) -> Self {
        Link(*account.public_key().as_bytes())
    }

    /// Create from a source block hash (for receive blocks).
    pub fn from_block_hash(hash: &BlockHash) -> Self {
        Link(*hash.as_bytes())
    }

    /// Create from a public key.
    pub fn from_public_key(key: &PublicKey) -> Self {
        Link(*key.as_bytes())
    }

    /// Try to interpret as a block hash.
    pub fn as_block_hash(&self) -> BlockHash {
        BlockHash(self.0)
    }

    /// Try to interpret as a public key.
    pub fn as_public_key(&self) -> PublicKey {
        PublicKey::from_bytes(self.0)
    }

    /// Convert to hex string.
    pub fn to_hex(&self) -> String {
        hex::encode_upper(self.0)
    }

    /// Create from hex string.
    pub fn from_hex(s: &str) -> Result<Self> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 32 {
            return Err(Error::InvalidBlockHash);
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Link(arr))
    }

    /// Check if this is the zero link.
    pub fn is_zero(&self) -> bool {
        self.0 == [0u8; 32]
    }
}

impl fmt::Debug for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Link({})", self.to_hex())
    }
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl From<[u8; 32]> for Link {
    fn from(bytes: [u8; 32]) -> Self {
        Link(bytes)
    }
}

impl From<BlockHash> for Link {
    fn from(hash: BlockHash) -> Self {
        Link(*hash.as_bytes())
    }
}

impl From<PublicKey> for Link {
    fn from(key: PublicKey) -> Self {
        Link(*key.as_bytes())
    }
}

impl Serialize for Link {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Link {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Link::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

/// Block subtype indicating the operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Subtype {
    /// Send funds to another account.
    Send,
    /// Receive funds from a pending block.
    Receive,
    /// Open account (first receive).
    Open,
    /// Change representative.
    Change,
    /// Epoch block (network upgrade).
    Epoch,
}

impl Subtype {
    /// Get the string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Subtype::Send => "send",
            Subtype::Receive => "receive",
            Subtype::Open => "open",
            Subtype::Change => "change",
            Subtype::Epoch => "epoch",
        }
    }
}

impl fmt::Display for Subtype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Nano state block.
///
/// State blocks are the only block type used in modern Nano.
/// They contain all information needed to represent any transaction type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateBlock {
    /// Block type (always "state").
    #[serde(rename = "type")]
    pub block_type: String,

    /// Account this block belongs to.
    pub account: Account,

    /// Hash of the previous block (zero for open blocks).
    pub previous: BlockHash,

    /// Representative account.
    pub representative: Account,

    /// Account balance after this block.
    pub balance: Raw,

    /// Link field (destination, source, or zero).
    pub link: Link,

    /// Ed25519 signature of the block hash.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<Signature>,

    /// Proof of work.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work: Option<Work>,

    /// Block subtype (send, receive, open, change, epoch).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<Subtype>,
}

impl StateBlock {
    /// Create a new state block.
    pub fn new(
        account: Account,
        previous: BlockHash,
        representative: Account,
        balance: Raw,
        link: Link,
    ) -> Self {
        StateBlock {
            block_type: "state".to_string(),
            account,
            previous,
            representative,
            balance,
            link,
            signature: None,
            work: None,
            subtype: None,
        }
    }

    /// Set the block subtype.
    pub fn with_subtype(mut self, subtype: Subtype) -> Self {
        self.subtype = Some(subtype);
        self
    }

    /// Set the signature.
    pub fn with_signature(mut self, signature: Signature) -> Self {
        self.signature = Some(signature);
        self
    }

    /// Set the work.
    pub fn with_work(mut self, work: Work) -> Self {
        self.work = Some(work);
        self
    }

    /// Check if the block is signed.
    pub fn is_signed(&self) -> bool {
        self.signature.is_some()
    }

    /// Check if the block has work.
    pub fn has_work(&self) -> bool {
        self.work.is_some()
    }

    /// Check if this is an open block (first block for an account).
    pub fn is_open(&self) -> bool {
        self.previous.is_zero()
    }

    /// Infer the subtype from block contents.
    pub fn infer_subtype(&self, previous_balance: Option<Raw>) -> Subtype {
        if self.previous.is_zero() {
            return Subtype::Open;
        }

        if self.link.is_zero() {
            return Subtype::Change;
        }

        match previous_balance {
            Some(prev) if self.balance < prev => Subtype::Send,
            Some(prev) if self.balance > prev => Subtype::Receive,
            Some(_) => Subtype::Change,
            None => {
                // Can't determine without previous balance
                // Default to change if link is zero, otherwise assume send
                if self.link.is_zero() {
                    Subtype::Change
                } else {
                    Subtype::Send
                }
            }
        }
    }
}

impl fmt::Display for StateBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "StateBlock {{ account: {}, previous: {}, balance: {} }}",
            self.account, self.previous, self.balance
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_HASH_HEX: &str = "991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948";

    #[test]
    fn test_block_hash_from_hex() {
        let hash = BlockHash::from_hex(TEST_HASH_HEX).unwrap();
        assert_eq!(hash.to_hex(), TEST_HASH_HEX);
    }

    #[test]
    fn test_block_hash_zero() {
        let zero = BlockHash::ZERO;
        assert!(zero.is_zero());
        assert_eq!(
            zero.to_hex(),
            "0000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_block_hash_roundtrip() {
        let bytes = [0xABu8; 32];
        let hash = BlockHash::from_bytes(bytes);
        assert_eq!(hash.as_bytes(), &bytes);

        let hex_str = hash.to_hex();
        let recovered = BlockHash::from_hex(&hex_str).unwrap();
        assert_eq!(hash, recovered);
    }

    #[test]
    fn test_link_from_account() {
        let pk =
            PublicKey::from_hex("E89208DD038FBB269987689621D52292AE9C35941A7484756ECCED92A65093BA")
                .unwrap();
        let account = Account::from_public_key(&pk);
        let link = Link::from_account(&account);

        assert_eq!(link.as_bytes(), pk.as_bytes());
    }

    #[test]
    fn test_link_from_block_hash() {
        let hash = BlockHash::from_hex(TEST_HASH_HEX).unwrap();
        let link = Link::from_block_hash(&hash);

        assert_eq!(link.as_block_hash(), hash);
    }

    #[test]
    fn test_subtype_display() {
        assert_eq!(Subtype::Send.to_string(), "send");
        assert_eq!(Subtype::Receive.to_string(), "receive");
        assert_eq!(Subtype::Open.to_string(), "open");
        assert_eq!(Subtype::Change.to_string(), "change");
        assert_eq!(Subtype::Epoch.to_string(), "epoch");
    }

    #[test]
    fn test_state_block_creation() {
        let pk =
            PublicKey::from_hex("E89208DD038FBB269987689621D52292AE9C35941A7484756ECCED92A65093BA")
                .unwrap();
        let account = Account::from_public_key(&pk);

        let block = StateBlock::new(
            account.clone(),
            BlockHash::ZERO,
            account.clone(),
            Raw::new(1000),
            Link::ZERO,
        );

        assert_eq!(block.block_type, "state");
        assert!(block.is_open());
        assert!(!block.is_signed());
        assert!(!block.has_work());
    }

    #[test]
    fn test_state_block_infer_subtype() {
        let pk =
            PublicKey::from_hex("E89208DD038FBB269987689621D52292AE9C35941A7484756ECCED92A65093BA")
                .unwrap();
        let account = Account::from_public_key(&pk);

        // Open block
        let block = StateBlock::new(
            account.clone(),
            BlockHash::ZERO,
            account.clone(),
            Raw::new(1000),
            Link::from_hex(TEST_HASH_HEX).unwrap(),
        );
        assert_eq!(block.infer_subtype(None), Subtype::Open);

        // Send block
        let block = StateBlock::new(
            account.clone(),
            BlockHash::from_hex(TEST_HASH_HEX).unwrap(),
            account.clone(),
            Raw::new(500),
            Link::from_public_key(&pk),
        );
        assert_eq!(block.infer_subtype(Some(Raw::new(1000))), Subtype::Send);

        // Receive block
        let block = StateBlock::new(
            account.clone(),
            BlockHash::from_hex(TEST_HASH_HEX).unwrap(),
            account.clone(),
            Raw::new(1500),
            Link::from_hex(TEST_HASH_HEX).unwrap(),
        );
        assert_eq!(block.infer_subtype(Some(Raw::new(1000))), Subtype::Receive);

        // Change block
        let block = StateBlock::new(
            account.clone(),
            BlockHash::from_hex(TEST_HASH_HEX).unwrap(),
            account.clone(),
            Raw::new(1000),
            Link::ZERO,
        );
        assert_eq!(block.infer_subtype(Some(Raw::new(1000))), Subtype::Change);
    }

    #[test]
    fn test_block_hash_serde() {
        let hash = BlockHash::from_hex(TEST_HASH_HEX).unwrap();
        let json = serde_json::to_string(&hash).unwrap();
        assert_eq!(json, format!("\"{}\"", TEST_HASH_HEX));

        let recovered: BlockHash = serde_json::from_str(&json).unwrap();
        assert_eq!(hash, recovered);
    }

    #[test]
    fn test_subtype_serde() {
        let subtype = Subtype::Send;
        let json = serde_json::to_string(&subtype).unwrap();
        assert_eq!(json, "\"send\"");

        let recovered: Subtype = serde_json::from_str(&json).unwrap();
        assert_eq!(subtype, recovered);
    }
}
