//! Block signing for Nano state blocks.

use crate::blocks::BlockHasher;
use crate::keys::KeyPair;
use crate::types::{BlockHash, PublicKey, Signature, StateBlock};

/// Block signer for signing and verifying blocks.
pub struct BlockSigner;

impl BlockSigner {
    /// Sign a state block and return the signature.
    ///
    /// The block is first hashed, then the hash is signed with the keypair.
    pub fn sign(block: &StateBlock, keypair: &KeyPair) -> Signature {
        let hash = BlockHasher::hash_state_block(block);
        keypair.sign(&hash)
    }

    /// Sign a block hash directly.
    pub fn sign_hash(hash: &BlockHash, keypair: &KeyPair) -> Signature {
        keypair.sign(hash)
    }

    /// Verify a block's signature.
    ///
    /// Returns true if the signature is valid for the block.
    pub fn verify(block: &StateBlock) -> bool {
        match &block.signature {
            Some(signature) => {
                let hash = BlockHasher::hash_state_block(block);
                KeyPair::verify_with_public_key(block.account.public_key(), &hash, signature)
            }
            None => false,
        }
    }

    /// Verify a signature against a block hash and public key.
    pub fn verify_hash(hash: &BlockHash, public_key: &PublicKey, signature: &Signature) -> bool {
        KeyPair::verify_with_public_key(public_key, hash, signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::Seed;
    use crate::types::{Link, Raw};

    fn test_keypair() -> KeyPair {
        let seed =
            Seed::from_hex("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap();
        seed.derive(0)
    }

    #[test]
    fn test_sign_block() {
        let keypair = test_keypair();
        let account = keypair.account();

        let block = StateBlock::new(
            account.clone(),
            BlockHash::ZERO,
            account.clone(),
            Raw::from_nano(1).unwrap(),
            Link::ZERO,
        );

        let signature = BlockSigner::sign(&block, &keypair);

        // Signature should be 64 bytes
        assert_eq!(signature.as_bytes().len(), 64);
    }

    #[test]
    fn test_sign_and_verify() {
        let keypair = test_keypair();
        let account = keypair.account();

        let mut block = StateBlock::new(
            account.clone(),
            BlockHash::ZERO,
            account.clone(),
            Raw::from_nano(1).unwrap(),
            Link::ZERO,
        );

        let signature = BlockSigner::sign(&block, &keypair);
        block.signature = Some(signature);

        assert!(BlockSigner::verify(&block));
    }

    #[test]
    fn test_verify_unsigned_block_fails() {
        let keypair = test_keypair();
        let account = keypair.account();

        let block = StateBlock::new(
            account.clone(),
            BlockHash::ZERO,
            account.clone(),
            Raw::from_nano(1).unwrap(),
            Link::ZERO,
        );

        assert!(!BlockSigner::verify(&block));
    }

    #[test]
    fn test_verify_tampered_block_fails() {
        let keypair = test_keypair();
        let account = keypair.account();

        let mut block = StateBlock::new(
            account.clone(),
            BlockHash::ZERO,
            account.clone(),
            Raw::from_nano(1).unwrap(),
            Link::ZERO,
        );

        let signature = BlockSigner::sign(&block, &keypair);
        block.signature = Some(signature);

        // Tamper with the block
        block.balance = Raw::from_nano(2).unwrap();

        assert!(!BlockSigner::verify(&block));
    }

    #[test]
    fn test_sign_hash_directly() {
        let keypair = test_keypair();
        let hash =
            BlockHash::from_hex("991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948")
                .unwrap();

        let signature = BlockSigner::sign_hash(&hash, &keypair);

        assert!(BlockSigner::verify_hash(
            &hash,
            keypair.public_key(),
            &signature
        ));
    }

    #[test]
    fn test_signature_is_deterministic() {
        let keypair = test_keypair();
        let account = keypair.account();

        let block = StateBlock::new(
            account.clone(),
            BlockHash::ZERO,
            account.clone(),
            Raw::from_nano(1).unwrap(),
            Link::ZERO,
        );

        let sig1 = BlockSigner::sign(&block, &keypair);
        let sig2 = BlockSigner::sign(&block, &keypair);

        // Ed25519 signatures should be deterministic
        assert_eq!(sig1, sig2);
    }
}
