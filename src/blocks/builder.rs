//! Fluent block builder for creating Nano state blocks.

use crate::blocks::{BlockHasher, BlockSigner};
use crate::error::{BlockError, Error, Result};
use crate::keys::KeyPair;
use crate::types::{Account, BlockHash, Link, Raw, Signature, StateBlock, Subtype, Work};

/// Builder for creating state blocks.
///
/// # Example
///
/// ```
/// use xno_connect::prelude::*;
/// use xno_connect::blocks::BlockBuilder;
///
/// # fn main() -> xno_connect::error::Result<()> {
/// let seed = Seed::from_hex("0000000000000000000000000000000000000000000000000000000000000000")?;
/// let keypair = seed.derive(0);
/// let my_account = keypair.account();
/// let previous_hash = BlockHash::ZERO;
/// let rep_account = my_account.clone();
/// let new_balance = Raw::from_nano(1)?;
/// let destination = Account::from_public_key(&PublicKey::ZERO);
///
/// let block = BlockBuilder::new()
///     .account(my_account)
///     .previous(previous_hash)
///     .representative(rep_account)
///     .balance(new_balance)
///     .link_as_account(&destination)
///     .sign(&keypair)
///     .build()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct BlockBuilder {
    account: Option<Account>,
    previous: Option<BlockHash>,
    representative: Option<Account>,
    balance: Option<Raw>,
    link: Option<Link>,
    subtype: Option<Subtype>,
    signature: Option<Signature>,
    work: Option<Work>,
}

impl BlockBuilder {
    /// Create a new block builder.
    pub fn new() -> Self {
        BlockBuilder::default()
    }

    /// Set the account that owns this block.
    pub fn account(mut self, account: Account) -> Self {
        self.account = Some(account);
        self
    }

    /// Set the previous block hash.
    ///
    /// Use `BlockHash::ZERO` for open blocks.
    pub fn previous(mut self, hash: BlockHash) -> Self {
        self.previous = Some(hash);
        self
    }

    /// Set the representative account.
    pub fn representative(mut self, account: Account) -> Self {
        self.representative = Some(account);
        self
    }

    /// Set the balance after this block.
    pub fn balance(mut self, balance: Raw) -> Self {
        self.balance = Some(balance);
        self
    }

    /// Set the link field.
    ///
    /// For sends: destination account's public key
    /// For receives: source block hash
    /// For changes: zero
    pub fn link(mut self, link: Link) -> Self {
        self.link = Some(link);
        self
    }

    /// Set the link field from a destination account (for send blocks).
    pub fn link_as_account(mut self, account: &Account) -> Self {
        self.link = Some(Link::from_account(account));
        self
    }

    /// Set the link field from a source block hash (for receive blocks).
    pub fn link_as_block(mut self, hash: &BlockHash) -> Self {
        self.link = Some(Link::from_block_hash(hash));
        self
    }

    /// Set the block subtype.
    pub fn subtype(mut self, subtype: Subtype) -> Self {
        self.subtype = Some(subtype);
        self
    }

    /// Set the proof of work.
    pub fn work(mut self, work: Work) -> Self {
        self.work = Some(work);
        self
    }

    /// Sign the block with the given keypair.
    ///
    /// This computes the block hash and signs it.
    pub fn sign(mut self, keypair: &KeyPair) -> Self {
        if let Ok(block) = self.clone().build_unsigned() {
            let signature = BlockSigner::sign(&block, keypair);
            self.signature = Some(signature);
        }
        self
    }

    /// Set a pre-computed signature.
    pub fn signature(mut self, signature: Signature) -> Self {
        self.signature = Some(signature);
        self
    }

    /// Build the block without signature or work.
    fn build_unsigned(&self) -> Result<StateBlock> {
        let account = self
            .account
            .clone()
            .ok_or(Error::InvalidBlock(BlockError::MissingField("account")))?;
        let previous = self
            .previous
            .ok_or(Error::InvalidBlock(BlockError::MissingField("previous")))?;
        let representative =
            self.representative
                .clone()
                .ok_or(Error::InvalidBlock(BlockError::MissingField(
                    "representative",
                )))?;
        let balance = self
            .balance
            .ok_or(Error::InvalidBlock(BlockError::MissingField("balance")))?;
        let link = self
            .link
            .ok_or(Error::InvalidBlock(BlockError::MissingField("link")))?;

        let mut block = StateBlock::new(account, previous, representative, balance, link);
        block.subtype = self.subtype;

        Ok(block)
    }

    /// Build the state block.
    ///
    /// Returns an error if any required fields are missing.
    pub fn build(self) -> Result<StateBlock> {
        let mut block = self.build_unsigned()?;
        block.signature = self.signature;
        block.work = self.work;
        Ok(block)
    }

    /// Get the hash of the block being built.
    ///
    /// Returns an error if required fields are missing.
    pub fn hash(&self) -> Result<BlockHash> {
        let block = self.build_unsigned()?;
        Ok(BlockHasher::hash_state_block(&block))
    }
}

/// Create a send block builder with common fields pre-set.
pub fn send_block_builder(
    account: Account,
    previous: BlockHash,
    representative: Account,
    new_balance: Raw,
    destination: &Account,
) -> BlockBuilder {
    BlockBuilder::new()
        .account(account)
        .previous(previous)
        .representative(representative)
        .balance(new_balance)
        .link_as_account(destination)
        .subtype(Subtype::Send)
}

/// Create a receive block builder with common fields pre-set.
pub fn receive_block_builder(
    account: Account,
    previous: BlockHash,
    representative: Account,
    new_balance: Raw,
    source_hash: &BlockHash,
) -> BlockBuilder {
    BlockBuilder::new()
        .account(account)
        .previous(previous)
        .representative(representative)
        .balance(new_balance)
        .link_as_block(source_hash)
        .subtype(Subtype::Receive)
}

/// Create an open block builder with common fields pre-set.
pub fn open_block_builder(
    account: Account,
    representative: Account,
    balance: Raw,
    source_hash: &BlockHash,
) -> BlockBuilder {
    BlockBuilder::new()
        .account(account)
        .previous(BlockHash::ZERO)
        .representative(representative)
        .balance(balance)
        .link_as_block(source_hash)
        .subtype(Subtype::Open)
}

/// Create a change block builder with common fields pre-set.
pub fn change_block_builder(
    account: Account,
    previous: BlockHash,
    new_representative: Account,
    balance: Raw,
) -> BlockBuilder {
    BlockBuilder::new()
        .account(account)
        .previous(previous)
        .representative(new_representative)
        .balance(balance)
        .link(Link::ZERO)
        .subtype(Subtype::Change)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::Seed;
    use crate::types::PublicKey;

    fn test_keypair() -> KeyPair {
        let seed =
            Seed::from_hex("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap();
        seed.derive(0)
    }

    #[test]
    fn test_block_builder_basic() {
        let keypair = test_keypair();
        let account = keypair.account();

        let block = BlockBuilder::new()
            .account(account.clone())
            .previous(BlockHash::ZERO)
            .representative(account.clone())
            .balance(Raw::from_nano(1).unwrap())
            .link(Link::ZERO)
            .build()
            .unwrap();

        assert_eq!(block.account, account);
        assert!(block.previous.is_zero());
        assert!(block.signature.is_none());
    }

    #[test]
    fn test_block_builder_with_signature() {
        let keypair = test_keypair();
        let account = keypair.account();

        let block = BlockBuilder::new()
            .account(account.clone())
            .previous(BlockHash::ZERO)
            .representative(account.clone())
            .balance(Raw::from_nano(1).unwrap())
            .link(Link::ZERO)
            .sign(&keypair)
            .build()
            .unwrap();

        assert!(block.signature.is_some());
        assert!(BlockSigner::verify(&block));
    }

    #[test]
    fn test_block_builder_with_precomputed_signature() {
        let keypair = test_keypair();
        let account = keypair.account();

        let block_builder = BlockBuilder::new()
            .account(account.clone())
            .previous(BlockHash::ZERO)
            .representative(account.clone())
            .balance(Raw::from_nano(1).unwrap())
            .link(Link::ZERO);

        let signature =
            BlockSigner::sign(&block_builder.clone().build_unsigned().unwrap(), &keypair);
        let block = block_builder.signature(signature).build().unwrap();

        assert!(block.signature.is_some());
        assert!(BlockSigner::verify(&block));
    }

    #[test]
    fn test_block_builder_missing_field() {
        let result = BlockBuilder::new()
            .account(Account::from_public_key(&PublicKey::ZERO))
            .build();

        assert!(matches!(
            result,
            Err(Error::InvalidBlock(BlockError::MissingField(_)))
        ));
    }

    #[test]
    fn test_send_block_builder() {
        let keypair = test_keypair();
        let account = keypair.account();
        let destination = Account::from_public_key(&PublicKey::ZERO);

        let block = send_block_builder(
            account.clone(),
            BlockHash::ZERO,
            account.clone(),
            Raw::new(500),
            &destination,
        )
        .sign(&keypair)
        .build()
        .unwrap();

        assert_eq!(block.subtype, Some(Subtype::Send));
        assert_eq!(block.link.as_public_key(), *destination.public_key());
    }

    #[test]
    fn test_receive_block_builder() {
        let keypair = test_keypair();
        let account = keypair.account();
        let source =
            BlockHash::from_hex("991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948")
                .unwrap();

        let block = receive_block_builder(
            account.clone(),
            BlockHash::ZERO,
            account.clone(),
            Raw::from_nano(1).unwrap(),
            &source,
        )
        .sign(&keypair)
        .build()
        .unwrap();

        assert_eq!(block.subtype, Some(Subtype::Receive));
        assert_eq!(block.link.as_block_hash(), source);
    }

    #[test]
    fn test_open_block_builder() {
        let keypair = test_keypair();
        let account = keypair.account();
        let source =
            BlockHash::from_hex("991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948")
                .unwrap();

        let block = open_block_builder(
            account.clone(),
            account.clone(),
            Raw::from_nano(1).unwrap(),
            &source,
        )
        .sign(&keypair)
        .build()
        .unwrap();

        assert_eq!(block.subtype, Some(Subtype::Open));
        assert!(block.previous.is_zero());
    }

    #[test]
    fn test_change_block_builder() {
        let keypair = test_keypair();
        let account = keypair.account();
        let new_rep = Account::from_public_key(&PublicKey::ZERO);

        let block = change_block_builder(
            account.clone(),
            BlockHash::ZERO,
            new_rep.clone(),
            Raw::from_nano(1).unwrap(),
        )
        .sign(&keypair)
        .build()
        .unwrap();

        assert_eq!(block.subtype, Some(Subtype::Change));
        assert!(block.link.is_zero());
        assert_eq!(block.representative, new_rep);
    }

    #[test]
    fn test_get_hash() {
        let keypair = test_keypair();
        let account = keypair.account();

        let builder = BlockBuilder::new()
            .account(account.clone())
            .previous(BlockHash::ZERO)
            .representative(account.clone())
            .balance(Raw::from_nano(1).unwrap())
            .link(Link::ZERO);

        let hash1 = builder.hash().unwrap();
        let block = builder.build().unwrap();
        let hash2 = BlockHasher::hash_state_block(&block);

        assert_eq!(hash1, hash2);
    }
}
