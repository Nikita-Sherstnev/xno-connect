//! Wallet account operations.

use crate::blocks::{
    create_change_block, create_open_block, create_receive_block, create_send_block, BlockBuilder,
};
use crate::keys::KeyPair;

#[cfg(feature = "rpc")]
use crate::error::Result;
use crate::types::{Account, BlockHash, Raw, StateBlock, Subtype, Work};
#[cfg(feature = "rpc")]
use alloc::vec::Vec;

#[cfg(feature = "rpc")]
use crate::rpc::RpcClient;

#[cfg(feature = "work-cpu")]
use crate::work::CpuWorkGenerator;

/// A single account within a wallet.
///
/// Provides high-level operations for a specific account.
pub struct WalletAccount {
    keypair: KeyPair,
    index: u32,
}

impl WalletAccount {
    /// Create a new wallet account.
    pub(crate) fn new(keypair: KeyPair, index: u32) -> Self {
        WalletAccount { keypair, index }
    }

    /// Get the account index.
    pub fn index(&self) -> u32 {
        self.index
    }

    /// Get the account address.
    pub fn address(&self) -> Account {
        self.keypair.account()
    }

    /// Get the keypair for signing.
    pub fn keypair(&self) -> &KeyPair {
        &self.keypair
    }

    // ==================== Block creation ====================

    /// Create a send block.
    ///
    /// # Arguments
    /// * `previous` - Hash of the previous block
    /// * `representative` - Current representative
    /// * `current_balance` - Balance before sending
    /// * `amount` - Amount to send
    /// * `destination` - Destination account
    /// * `work` - Optional proof of work
    pub fn create_send(
        &self,
        previous: BlockHash,
        representative: Account,
        current_balance: Raw,
        amount: Raw,
        destination: &Account,
        work: Option<Work>,
    ) -> StateBlock {
        create_send_block(
            &self.keypair,
            previous,
            representative,
            current_balance,
            amount,
            destination,
            work,
        )
    }

    /// Create a receive block.
    ///
    /// # Arguments
    /// * `previous` - Hash of the previous block
    /// * `representative` - Current representative
    /// * `current_balance` - Balance before receiving
    /// * `amount` - Amount being received
    /// * `source_hash` - Hash of the send block
    /// * `work` - Optional proof of work
    pub fn create_receive(
        &self,
        previous: BlockHash,
        representative: Account,
        current_balance: Raw,
        amount: Raw,
        source_hash: &BlockHash,
        work: Option<Work>,
    ) -> StateBlock {
        create_receive_block(
            &self.keypair,
            previous,
            representative,
            current_balance,
            amount,
            source_hash,
            work,
        )
    }

    /// Create an open block (first receive).
    ///
    /// # Arguments
    /// * `representative` - Representative for the account
    /// * `amount` - Amount being received
    /// * `source_hash` - Hash of the send block
    /// * `work` - Optional proof of work
    pub fn create_open(
        &self,
        representative: Account,
        amount: Raw,
        source_hash: &BlockHash,
        work: Option<Work>,
    ) -> StateBlock {
        create_open_block(&self.keypair, representative, amount, source_hash, work)
    }

    /// Create a change block.
    ///
    /// # Arguments
    /// * `previous` - Hash of the previous block
    /// * `new_representative` - New representative account
    /// * `balance` - Current balance (unchanged)
    /// * `work` - Optional proof of work
    pub fn create_change(
        &self,
        previous: BlockHash,
        new_representative: Account,
        balance: Raw,
        work: Option<Work>,
    ) -> StateBlock {
        create_change_block(&self.keypair, previous, new_representative, balance, work)
    }

    /// Create a send block that also changes the representative.
    ///
    /// This combines a send and representative change into a single block.
    ///
    /// # Arguments
    /// * `previous` - Hash of the previous block
    /// * `new_representative` - New representative account
    /// * `current_balance` - Balance before sending
    /// * `amount` - Amount to send
    /// * `destination` - Destination account
    /// * `work` - Optional proof of work
    pub fn create_send_and_change(
        &self,
        previous: BlockHash,
        new_representative: Account,
        current_balance: Raw,
        amount: Raw,
        destination: &Account,
        work: Option<Work>,
    ) -> StateBlock {
        let new_balance = current_balance.checked_sub(amount).unwrap_or(Raw::ZERO);

        let mut builder = BlockBuilder::new()
            .account(self.keypair.account())
            .previous(previous)
            .representative(new_representative)
            .balance(new_balance)
            .link_as_account(destination)
            .subtype(Subtype::Send)
            .sign(&self.keypair);

        if let Some(w) = work {
            builder = builder.work(w);
        }

        builder.build().expect("all fields provided")
    }

    // ==================== Local work generation methods ====================

    /// Generate work locally using CPU.
    #[cfg(feature = "work-cpu")]
    fn generate_work(&self, hash: &BlockHash, subtype: Subtype) -> Result<Work> {
        let generator = CpuWorkGenerator::new();
        generator.generate_for_subtype(hash, subtype)
    }

    // ==================== RPC-dependent methods ====================

    /// Get the account balance.
    #[cfg(feature = "rpc")]
    pub async fn balance(&self, client: &RpcClient) -> Result<crate::rpc::AccountBalanceResponse> {
        client.account_balance(&self.address()).await
    }

    /// Get account info.
    #[cfg(feature = "rpc")]
    pub async fn info(&self, client: &RpcClient) -> Result<crate::rpc::AccountInfoResponse> {
        client.account_info(&self.address()).await
    }

    /// Get account history.
    #[cfg(feature = "rpc")]
    pub async fn history(
        &self,
        count: u64,
        client: &RpcClient,
    ) -> Result<crate::rpc::AccountHistoryResponse> {
        client.account_history(&self.address(), count).await
    }

    /// Get receivable blocks.
    #[cfg(feature = "rpc")]
    pub async fn receivable(
        &self,
        count: u64,
        client: &RpcClient,
    ) -> Result<crate::rpc::AccountsReceivableResponse> {
        client.accounts_receivable(&[self.address()], count).await
    }

    /// Process (submit) a block to the network.
    #[cfg(feature = "rpc")]
    pub async fn process(
        &self,
        block: StateBlock,
        client: &RpcClient,
    ) -> Result<crate::rpc::ProcessResponse> {
        client.process(block).await
    }

    /// Send Nano to another account.
    ///
    /// This is a high-level method that:
    /// 1. Gets the current account info
    /// 2. Creates a send block
    /// 3. Generates work (via the node)
    /// 4. Submits the block
    ///
    /// # Arguments
    /// * `destination` - Destination account
    /// * `amount` - Amount to send
    /// * `client` - RPC client
    #[cfg(feature = "rpc")]
    pub async fn send(
        &self,
        destination: &Account,
        amount: Raw,
        client: &RpcClient,
    ) -> Result<crate::rpc::ProcessResponse> {
        // Get account info
        let info = self.info(client).await?;

        // Generate work
        let work_response = client.work_generate(&info.frontier).await?;

        // Create and sign the block
        let block = self.create_send(
            info.frontier,
            info.representative.unwrap_or_else(|| self.address()),
            info.balance,
            amount,
            destination,
            Some(work_response.work),
        );

        // Submit the block
        client.process(block).await
    }

    /// Change representative.
    ///
    /// # Arguments
    /// * `new_representative` - New representative account
    /// * `client` - RPC client
    #[cfg(feature = "rpc")]
    pub async fn change_representative(
        &self,
        new_representative: &Account,
        client: &RpcClient,
    ) -> Result<crate::rpc::ProcessResponse> {
        // Get account info
        let info = self.info(client).await?;

        // Generate work
        let work_response = client.work_generate(&info.frontier).await?;

        // Create and sign the block
        let block = self.create_change(
            info.frontier,
            new_representative.clone(),
            info.balance,
            Some(work_response.work),
        );

        // Submit the block
        client.process(block).await
    }

    /// Receive a pending block.
    ///
    /// # Arguments
    /// * `source_hash` - Hash of the send block to receive
    /// * `amount` - Amount being received
    /// * `client` - RPC client
    #[cfg(feature = "rpc")]
    pub async fn receive(
        &self,
        source_hash: &BlockHash,
        amount: Raw,
        client: &RpcClient,
    ) -> Result<crate::rpc::ProcessResponse> {
        // Try to get account info (may fail if account doesn't exist yet)
        let info_result = self.info(client).await;

        match info_result {
            Ok(info) => {
                // Existing account - create receive block
                let work_response = client.work_generate(&info.frontier).await?;
                let block = self.create_receive(
                    info.frontier,
                    info.representative.unwrap_or_else(|| self.address()),
                    info.balance,
                    amount,
                    source_hash,
                    Some(work_response.work),
                );
                client.process(block).await
            }
            Err(_) => {
                // New account - create open block
                // For open blocks, work is computed on the account's public key
                let pub_key_hash = BlockHash::from_bytes(*self.keypair.public_key().as_bytes());
                let work_response = client.work_generate(&pub_key_hash).await?;
                let block = self.create_open(
                    self.address(),
                    amount,
                    source_hash,
                    Some(work_response.work),
                );
                client.process(block).await
            }
        }
    }

    /// Receive all pending blocks.
    ///
    /// Returns the list of processed block hashes.
    ///
    /// # Arguments
    /// * `client` - RPC client
    #[cfg(feature = "rpc")]
    pub async fn receive_all(&self, client: &RpcClient) -> Result<Vec<BlockHash>> {
        let mut received = Vec::new();

        // Get receivable blocks
        let receivable = self.receivable(100, client).await?;
        let account_key = self.address().to_string();

        if let Some(blocks) = receivable.blocks.get(&account_key) {
            // Parse the receivable blocks
            if let Some(obj) = blocks.as_object() {
                for (hash_str, value) in obj {
                    let source_hash = BlockHash::from_hex(hash_str)?;
                    let amount = if let Some(amount_str) = value.as_str() {
                        amount_str.parse::<Raw>()?
                    } else if let Some(obj) = value.as_object() {
                        if let Some(amount_str) = obj.get("amount").and_then(|v| v.as_str()) {
                            amount_str.parse::<Raw>()?
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    };

                    let response = self.receive(&source_hash, amount, client).await?;
                    received.push(response.hash);
                }
            } else if let Some(arr) = blocks.as_array() {
                // Simple list of hashes (need to get amounts separately)
                for hash_val in arr {
                    if let Some(hash_str) = hash_val.as_str() {
                        let source_hash = BlockHash::from_hex(hash_str)?;
                        // Get block info to find the amount
                        let block_info = client.block_info(&source_hash).await?;
                        let response = self
                            .receive(&source_hash, block_info.amount, client)
                            .await?;
                        received.push(response.hash);
                    }
                }
            }
        }

        Ok(received)
    }

    /// Send and change representative in one block.
    ///
    /// # Arguments
    /// * `destination` - Destination account
    /// * `amount` - Amount to send
    /// * `new_representative` - New representative account
    /// * `client` - RPC client
    #[cfg(feature = "rpc")]
    pub async fn send_and_change(
        &self,
        destination: &Account,
        amount: Raw,
        new_representative: &Account,
        client: &RpcClient,
    ) -> Result<crate::rpc::ProcessResponse> {
        // Get account info
        let info = self.info(client).await?;

        // Generate work
        let work_response = client.work_generate(&info.frontier).await?;

        // Create and sign the block
        let block = self.create_send_and_change(
            info.frontier,
            new_representative.clone(),
            info.balance,
            amount,
            destination,
            Some(work_response.work),
        );

        // Submit the block
        client.process(block).await
    }

    // ==================== Local work generation variants ====================

    /// Send Nano using local CPU work generation.
    #[cfg(all(feature = "rpc", feature = "work-cpu", not(target_arch = "wasm32")))]
    pub async fn send_local(
        &self,
        destination: &Account,
        amount: Raw,
        client: &RpcClient,
    ) -> Result<crate::rpc::ProcessResponse> {
        let info = self.info(client).await?;
        let work = self.generate_work(&info.frontier, Subtype::Send)?;
        let block = self.create_send(
            info.frontier,
            info.representative.unwrap_or_else(|| self.address()),
            info.balance,
            amount,
            destination,
            Some(work),
        );
        client.process(block).await
    }

    /// Receive a pending block using local CPU work generation.
    #[cfg(all(feature = "rpc", feature = "work-cpu", not(target_arch = "wasm32")))]
    pub async fn receive_local(
        &self,
        source_hash: &BlockHash,
        amount: Raw,
        client: &RpcClient,
    ) -> Result<crate::rpc::ProcessResponse> {
        let info_result = self.info(client).await;

        match info_result {
            Ok(info) => {
                let work = self.generate_work(&info.frontier, Subtype::Receive)?;
                let block = self.create_receive(
                    info.frontier,
                    info.representative.unwrap_or_else(|| self.address()),
                    info.balance,
                    amount,
                    source_hash,
                    Some(work),
                );
                client.process(block).await
            }
            Err(_) => {
                // For open blocks, work is on public key
                let pub_key_hash = BlockHash::from_bytes(*self.keypair.public_key().as_bytes());
                let work = self.generate_work(&pub_key_hash, Subtype::Open)?;
                let block = self.create_open(self.address(), amount, source_hash, Some(work));
                client.process(block).await
            }
        }
    }

    /// Receive all pending blocks using local CPU work generation.
    #[cfg(all(feature = "rpc", feature = "work-cpu", not(target_arch = "wasm32")))]
    pub async fn receive_all_local(&self, client: &RpcClient) -> Result<Vec<BlockHash>> {
        let mut received = Vec::new();

        let receivable = self.receivable(100, client).await?;
        let account_key = self.address().to_string();

        if let Some(blocks) = receivable.blocks.get(&account_key) {
            if let Some(obj) = blocks.as_object() {
                for (hash_str, value) in obj {
                    let source_hash = BlockHash::from_hex(hash_str)?;
                    let amount = if let Some(amount_str) = value.as_str() {
                        amount_str.parse::<Raw>()?
                    } else if let Some(obj) = value.as_object() {
                        if let Some(amount_str) = obj.get("amount").and_then(|v| v.as_str()) {
                            amount_str.parse::<Raw>()?
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    };

                    let response = self.receive_local(&source_hash, amount, client).await?;
                    received.push(response.hash);
                }
            }
        }

        Ok(received)
    }

    /// Change representative using local CPU work generation.
    #[cfg(all(feature = "rpc", feature = "work-cpu", not(target_arch = "wasm32")))]
    pub async fn change_representative_local(
        &self,
        new_representative: &Account,
        client: &RpcClient,
    ) -> Result<crate::rpc::ProcessResponse> {
        let info = self.info(client).await?;
        let work = self.generate_work(&info.frontier, Subtype::Change)?;
        let block = self.create_change(
            info.frontier,
            new_representative.clone(),
            info.balance,
            Some(work),
        );
        client.process(block).await
    }

    /// Send and change representative using local CPU work generation.
    #[cfg(all(feature = "rpc", feature = "work-cpu", not(target_arch = "wasm32")))]
    pub async fn send_and_change_local(
        &self,
        destination: &Account,
        amount: Raw,
        new_representative: &Account,
        client: &RpcClient,
    ) -> Result<crate::rpc::ProcessResponse> {
        let info = self.info(client).await?;
        let work = self.generate_work(&info.frontier, Subtype::Send)?;
        let block = self.create_send_and_change(
            info.frontier,
            new_representative.clone(),
            info.balance,
            amount,
            destination,
            Some(work),
        );
        client.process(block).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::Seed;
    use crate::types::PublicKey;

    fn test_account() -> WalletAccount {
        let seed =
            Seed::from_hex("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap();
        let keypair = seed.derive(0);
        WalletAccount::new(keypair, 0)
    }

    #[test]
    fn test_wallet_account_creation() {
        let account = test_account();
        assert_eq!(account.index(), 0);
        assert_eq!(
            account.address().as_str(),
            "nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7"
        );
    }

    #[test]
    fn test_create_send() {
        let account = test_account();
        let destination = Account::from_public_key(&PublicKey::ZERO);

        let block = account.create_send(
            BlockHash::from_hex("991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948")
                .unwrap(),
            account.address(),
            Raw::from_nano(10).unwrap(),
            Raw::from_nano(3).unwrap(),
            &destination,
            None,
        );

        assert!(block.signature.is_some());
        assert_eq!(block.balance, Raw::from_nano(7).unwrap());
    }

    #[test]
    fn test_create_receive() {
        let account = test_account();
        let source =
            BlockHash::from_hex("991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948")
                .unwrap();

        let block = account.create_receive(
            BlockHash::from_hex("0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap(),
            account.address(),
            Raw::from_nano(5).unwrap(),
            Raw::from_nano(3).unwrap(),
            &source,
            None,
        );

        assert!(block.signature.is_some());
        assert_eq!(block.balance, Raw::from_nano(8).unwrap());
    }

    #[test]
    fn test_create_open() {
        let account = test_account();
        let source =
            BlockHash::from_hex("991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948")
                .unwrap();

        let block = account.create_open(
            account.address(),
            Raw::from_nano(10).unwrap(),
            &source,
            None,
        );

        assert!(block.signature.is_some());
        assert!(block.previous.is_zero());
        assert_eq!(block.balance, Raw::from_nano(10).unwrap());
    }

    #[test]
    fn test_create_change() {
        let account = test_account();
        let new_rep = Account::from_public_key(&PublicKey::ZERO);

        let block = account.create_change(
            BlockHash::from_hex("991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948")
                .unwrap(),
            new_rep.clone(),
            Raw::from_nano(10).unwrap(),
            None,
        );

        assert!(block.signature.is_some());
        assert!(block.link.is_zero());
        assert_eq!(block.representative, new_rep);
    }

    #[test]
    fn test_create_send_and_change() {
        let account = test_account();
        let destination = Account::from_public_key(&PublicKey::ZERO);
        let new_rep = Account::from_public_key(
            &PublicKey::from_hex(
                "0000000000000000000000000000000000000000000000000000000000000001",
            )
            .unwrap(),
        );

        let block = account.create_send_and_change(
            BlockHash::from_hex("991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948")
                .unwrap(),
            new_rep.clone(),
            Raw::from_nano(10).unwrap(),
            Raw::from_nano(3).unwrap(),
            &destination,
            None,
        );

        // Verify it's a send block with changed representative
        assert!(block.signature.is_some());
        assert_eq!(block.balance, Raw::from_nano(7).unwrap()); // 10 - 3
        assert_eq!(block.representative, new_rep);
        assert_eq!(block.subtype, Some(Subtype::Send));
        // Link should be destination's public key
        assert_eq!(block.link.as_public_key(), *destination.public_key());
    }

    #[test]
    fn test_block_signatures_are_valid() {
        use crate::blocks::BlockSigner;

        let account = test_account();
        let destination = Account::from_public_key(&PublicKey::ZERO);
        let source =
            BlockHash::from_hex("991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948")
                .unwrap();
        let new_rep = Account::from_public_key(
            &PublicKey::from_hex(
                "0000000000000000000000000000000000000000000000000000000000000001",
            )
            .unwrap(),
        );

        // Test send block signature
        let send_block = account.create_send(
            source,
            account.address(),
            Raw::from_nano(10).unwrap(),
            Raw::from_nano(3).unwrap(),
            &destination,
            None,
        );
        assert!(
            BlockSigner::verify(&send_block),
            "Send block signature invalid"
        );

        // Test receive block signature
        let receive_block = account.create_receive(
            source,
            account.address(),
            Raw::from_nano(5).unwrap(),
            Raw::from_nano(3).unwrap(),
            &source,
            None,
        );
        assert!(
            BlockSigner::verify(&receive_block),
            "Receive block signature invalid"
        );

        // Test open block signature
        let open_block = account.create_open(
            account.address(),
            Raw::from_nano(10).unwrap(),
            &source,
            None,
        );
        assert!(
            BlockSigner::verify(&open_block),
            "Open block signature invalid"
        );

        // Test change block signature
        let change_block =
            account.create_change(source, new_rep.clone(), Raw::from_nano(10).unwrap(), None);
        assert!(
            BlockSigner::verify(&change_block),
            "Change block signature invalid"
        );

        // Test send_and_change block signature
        let send_change_block = account.create_send_and_change(
            source,
            new_rep,
            Raw::from_nano(10).unwrap(),
            Raw::from_nano(3).unwrap(),
            &destination,
            None,
        );
        assert!(
            BlockSigner::verify(&send_change_block),
            "Send+change block signature invalid"
        );
    }
}
