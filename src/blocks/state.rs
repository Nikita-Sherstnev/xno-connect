//! Convenience functions for creating state blocks.

use crate::blocks::builder::BlockBuilder;
use crate::keys::KeyPair;
use crate::types::{Account, BlockHash, Link, Raw, StateBlock, Subtype, Work};

/// Create a send block.
///
/// # Arguments
/// * `keypair` - The keypair for signing
/// * `previous` - Hash of the previous block
/// * `representative` - Current representative
/// * `current_balance` - Balance before this transaction
/// * `amount` - Amount to send
/// * `destination` - Destination account
/// * `work` - Optional proof of work
///
/// # Returns
/// A signed send block with the new balance (current - amount).
pub fn create_send_block(
    keypair: &KeyPair,
    previous: BlockHash,
    representative: Account,
    current_balance: Raw,
    amount: Raw,
    destination: &Account,
    work: Option<Work>,
) -> StateBlock {
    let new_balance = current_balance.checked_sub(amount).unwrap_or(Raw::ZERO);

    let mut builder = BlockBuilder::new()
        .account(keypair.account())
        .previous(previous)
        .representative(representative)
        .balance(new_balance)
        .link_as_account(destination)
        .subtype(Subtype::Send)
        .sign(keypair);

    if let Some(w) = work {
        builder = builder.work(w);
    }

    builder.build().expect("all fields provided")
}

/// Create a receive block.
///
/// # Arguments
/// * `keypair` - The keypair for signing
/// * `previous` - Hash of the previous block
/// * `representative` - Current representative
/// * `current_balance` - Balance before this transaction
/// * `amount` - Amount being received
/// * `source_hash` - Hash of the send block
/// * `work` - Optional proof of work
///
/// # Returns
/// A signed receive block with the new balance (current + amount).
pub fn create_receive_block(
    keypair: &KeyPair,
    previous: BlockHash,
    representative: Account,
    current_balance: Raw,
    amount: Raw,
    source_hash: &BlockHash,
    work: Option<Work>,
) -> StateBlock {
    let new_balance = current_balance.checked_add(amount).unwrap_or(Raw::MAX);

    let mut builder = BlockBuilder::new()
        .account(keypair.account())
        .previous(previous)
        .representative(representative)
        .balance(new_balance)
        .link_as_block(source_hash)
        .subtype(Subtype::Receive)
        .sign(keypair);

    if let Some(w) = work {
        builder = builder.work(w);
    }

    builder.build().expect("all fields provided")
}

/// Create an open block (first receive for a new account).
///
/// # Arguments
/// * `keypair` - The keypair for the new account
/// * `representative` - Representative for the new account
/// * `amount` - Amount being received
/// * `source_hash` - Hash of the send block
/// * `work` - Optional proof of work
///
/// # Returns
/// A signed open block.
pub fn create_open_block(
    keypair: &KeyPair,
    representative: Account,
    amount: Raw,
    source_hash: &BlockHash,
    work: Option<Work>,
) -> StateBlock {
    let mut builder = BlockBuilder::new()
        .account(keypair.account())
        .previous(BlockHash::ZERO)
        .representative(representative)
        .balance(amount)
        .link_as_block(source_hash)
        .subtype(Subtype::Open)
        .sign(keypair);

    if let Some(w) = work {
        builder = builder.work(w);
    }

    builder.build().expect("all fields provided")
}

/// Create a change block (change representative).
///
/// # Arguments
/// * `keypair` - The keypair for signing
/// * `previous` - Hash of the previous block
/// * `new_representative` - New representative account
/// * `balance` - Current balance (unchanged)
/// * `work` - Optional proof of work
///
/// # Returns
/// A signed change block.
pub fn create_change_block(
    keypair: &KeyPair,
    previous: BlockHash,
    new_representative: Account,
    balance: Raw,
    work: Option<Work>,
) -> StateBlock {
    let mut builder = BlockBuilder::new()
        .account(keypair.account())
        .previous(previous)
        .representative(new_representative)
        .balance(balance)
        .link(Link::ZERO)
        .subtype(Subtype::Change)
        .sign(keypair);

    if let Some(w) = work {
        builder = builder.work(w);
    }

    builder.build().expect("all fields provided")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::BlockSigner;
    use crate::keys::Seed;
    use crate::types::PublicKey;

    fn test_keypair() -> KeyPair {
        let seed =
            Seed::from_hex("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap();
        seed.derive(0)
    }

    #[test]
    fn test_create_send_block() {
        let keypair = test_keypair();
        let destination = Account::from_public_key(&PublicKey::ZERO);

        let block = create_send_block(
            &keypair,
            BlockHash::from_hex("991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948")
                .unwrap(),
            keypair.account(),
            Raw::from_nano(10).unwrap(),
            Raw::from_nano(3).unwrap(),
            &destination,
            None,
        );

        assert_eq!(block.subtype, Some(Subtype::Send));
        assert_eq!(block.balance, Raw::from_nano(7).unwrap());
        assert!(BlockSigner::verify(&block));
    }

    #[test]
    fn test_create_receive_block() {
        let keypair = test_keypair();
        let source =
            BlockHash::from_hex("991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948")
                .unwrap();

        let block = create_receive_block(
            &keypair,
            BlockHash::from_hex("0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap(),
            keypair.account(),
            Raw::from_nano(5).unwrap(),
            Raw::from_nano(3).unwrap(),
            &source,
            None,
        );

        assert_eq!(block.subtype, Some(Subtype::Receive));
        assert_eq!(block.balance, Raw::from_nano(8).unwrap());
        assert!(BlockSigner::verify(&block));
    }

    #[test]
    fn test_create_open_block() {
        let keypair = test_keypair();
        let source =
            BlockHash::from_hex("991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948")
                .unwrap();

        let block = create_open_block(
            &keypair,
            keypair.account(),
            Raw::from_nano(10).unwrap(),
            &source,
            None,
        );

        assert_eq!(block.subtype, Some(Subtype::Open));
        assert!(block.previous.is_zero());
        assert_eq!(block.balance, Raw::from_nano(10).unwrap());
        assert!(BlockSigner::verify(&block));
    }

    #[test]
    fn test_create_change_block() {
        let keypair = test_keypair();
        let new_rep = Account::from_public_key(&PublicKey::ZERO);

        let block = create_change_block(
            &keypair,
            BlockHash::from_hex("991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948")
                .unwrap(),
            new_rep.clone(),
            Raw::from_nano(10).unwrap(),
            None,
        );

        assert_eq!(block.subtype, Some(Subtype::Change));
        assert!(block.link.is_zero());
        assert_eq!(block.representative, new_rep);
        assert!(BlockSigner::verify(&block));
    }

    #[test]
    fn test_send_with_work() {
        let keypair = test_keypair();
        let destination = Account::from_public_key(&PublicKey::ZERO);

        let work = Work::from_hex("7202df8a7c380578").unwrap();
        let block = create_send_block(
            &keypair,
            BlockHash::ZERO,
            keypair.account(),
            Raw::from_nano(10).unwrap(),
            Raw::from_nano(3).unwrap(),
            &destination,
            Some(work),
        );

        assert!(block.work.is_some());
        assert_eq!(block.work.unwrap(), work);
    }
}
