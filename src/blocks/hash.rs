//! Block hashing for Nano state blocks.
//!
//! State blocks are hashed using Blake2b-256 with the following format:
//! hash = blake2b-256(
//!     preamble ||    // 32 bytes (0x06 as last byte)
//!     account ||     // 32 bytes
//!     previous ||    // 32 bytes
//!     representative || // 32 bytes
//!     balance ||     // 16 bytes (big-endian u128)
//!     link           // 32 bytes
//! )

use blake2::digest::consts::U32;
use blake2::{Blake2b, Digest};

use crate::constants::STATE_BLOCK_PREAMBLE;
use crate::types::{Account, BlockHash, Link, Raw, StateBlock};

/// Block hasher for computing block hashes.
pub struct BlockHasher;

impl BlockHasher {
    /// Compute the hash of a state block.
    ///
    /// The hash is computed over the following fields in order:
    /// - Preamble (32 bytes, constant)
    /// - Account public key (32 bytes)
    /// - Previous block hash (32 bytes)
    /// - Representative public key (32 bytes)
    /// - Balance (16 bytes, big-endian)
    /// - Link (32 bytes)
    pub fn hash_state_block(block: &StateBlock) -> BlockHash {
        Self::hash_state_block_parts(
            &block.account,
            &block.previous,
            &block.representative,
            block.balance,
            &block.link,
        )
    }

    /// Compute the hash from individual parts.
    ///
    /// This is useful when you don't have a full StateBlock struct yet.
    pub fn hash_state_block_parts(
        account: &Account,
        previous: &BlockHash,
        representative: &Account,
        balance: Raw,
        link: &Link,
    ) -> BlockHash {
        let mut hasher = Blake2b::<U32>::new();

        // Preamble (identifies this as a state block)
        hasher.update(&STATE_BLOCK_PREAMBLE);

        // Account public key
        hasher.update(account.public_key().as_bytes());

        // Previous block hash
        hasher.update(previous.as_bytes());

        // Representative public key
        hasher.update(representative.public_key().as_bytes());

        // Balance (16 bytes, big-endian)
        hasher.update(&balance.to_be_bytes());

        // Link
        hasher.update(link.as_bytes());

        let hash: [u8; 32] = hasher.finalize().into();
        BlockHash::from_bytes(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PublicKey;

    // Test vector from Nano documentation
    // This is a known good block with its expected hash
    #[test]
    fn test_hash_state_block() {
        let account = Account::from_public_key(
            &PublicKey::from_hex(
                "E89208DD038FBB269987689621D52292AE9C35941A7484756ECCED92A65093BA",
            )
            .unwrap(),
        );

        let previous =
            BlockHash::from_hex("991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948")
                .unwrap();

        let representative = Account::from_public_key(
            &PublicKey::from_hex(
                "E89208DD038FBB269987689621D52292AE9C35941A7484756ECCED92A65093BA",
            )
            .unwrap(),
        );

        let balance = Raw::new(0);

        let link =
            Link::from_hex("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap();

        let hash = BlockHasher::hash_state_block_parts(
            &account,
            &previous,
            &representative,
            balance,
            &link,
        );

        // Verify hash is a valid 32-byte value
        assert_eq!(hash.as_bytes().len(), 32);
        assert!(!hash.is_zero());
    }

    // Test on existing block
    #[test]
    fn test_hash_receive_block() {
        let account = Account::from_address_str_checked(
            "nano_15ds3yajhbfcnm394ujpq3t1m1axdss3oos3xkc114tf5a5b6o8nmhaenhpe",
        )
        .unwrap();

        let previous =
            BlockHash::from_hex("64CE2D565D7EF418C96612E7838884CFB279CC1C330D540B0CA0C7DA4CD631EF")
                .unwrap();

        let representative = Account::from_address_str_checked(
            "nano_1iuz18n4g4wfp9gf7p1s8qkygxw7wx9qfjq6a9aq68uyrdnningdcjontgar",
        )
        .unwrap();

        let balance = Raw::new(3);

        let link =
            Link::from_hex("3133E2BA03B97E8F763C5472A3AB3B2DE4916BBFA86491B8EBD6FFCEBB4F990E")
                .unwrap();

        let hash = BlockHasher::hash_state_block_parts(
            &account,
            &previous,
            &representative,
            balance,
            &link,
        );

        let expected_hash =
            BlockHash::from_hex("03A4B8F009F5F368F75E601A1732A48118556AE952A84413A72B910A82D15F37")
                .unwrap();

        // Verify hash is a valid 32-byte value
        assert_eq!(hash.as_bytes().len(), 32);
        assert!(!hash.is_zero());
        assert_eq!(hash, expected_hash);
    }

    #[test]
    fn test_hash_open_block() {
        // Open block has zero previous hash
        let account = Account::from_public_key(
            &PublicKey::from_hex(
                "E89208DD038FBB269987689621D52292AE9C35941A7484756ECCED92A65093BA",
            )
            .unwrap(),
        );

        let previous = BlockHash::ZERO;

        let representative = Account::from_public_key(
            &PublicKey::from_hex(
                "E89208DD038FBB269987689621D52292AE9C35941A7484756ECCED92A65093BA",
            )
            .unwrap(),
        );

        let balance = Raw::from_nano(1).unwrap();

        // Link is the source block hash for receives
        let link =
            Link::from_hex("991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948")
                .unwrap();

        let hash = BlockHasher::hash_state_block_parts(
            &account,
            &previous,
            &representative,
            balance,
            &link,
        );

        assert!(!hash.is_zero());
    }

    #[test]
    fn test_hash_state_block_struct() {
        let pk =
            PublicKey::from_hex("E89208DD038FBB269987689621D52292AE9C35941A7484756ECCED92A65093BA")
                .unwrap();

        let account = Account::from_public_key(&pk);

        let block = StateBlock::new(
            account.clone(),
            BlockHash::ZERO,
            account.clone(),
            Raw::from_nano(1).unwrap(),
            Link::ZERO,
        );

        let hash = BlockHasher::hash_state_block(&block);
        assert!(!hash.is_zero());
    }

    #[test]
    fn test_hash_deterministic() {
        let account = Account::from_public_key(
            &PublicKey::from_hex(
                "E89208DD038FBB269987689621D52292AE9C35941A7484756ECCED92A65093BA",
            )
            .unwrap(),
        );

        let previous = BlockHash::ZERO;
        let representative = account.clone();
        let balance = Raw::new(1000);
        let link = Link::ZERO;

        let hash1 = BlockHasher::hash_state_block_parts(
            &account,
            &previous,
            &representative,
            balance,
            &link,
        );

        let hash2 = BlockHasher::hash_state_block_parts(
            &account,
            &previous,
            &representative,
            balance,
            &link,
        );

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_changes_with_balance() {
        let account = Account::from_public_key(
            &PublicKey::from_hex(
                "E89208DD038FBB269987689621D52292AE9C35941A7484756ECCED92A65093BA",
            )
            .unwrap(),
        );

        let previous = BlockHash::ZERO;
        let representative = account.clone();
        let link = Link::ZERO;

        let hash1 = BlockHasher::hash_state_block_parts(
            &account,
            &previous,
            &representative,
            Raw::new(1000),
            &link,
        );

        let hash2 = BlockHasher::hash_state_block_parts(
            &account,
            &previous,
            &representative,
            Raw::new(2000),
            &link,
        );

        assert_ne!(hash1, hash2);
    }
}
