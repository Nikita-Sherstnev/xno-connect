//! High-level wallet implementation.

use alloc::vec::Vec;

use crate::error::Result;
use crate::keys::{KeyPair, Seed};
use crate::types::Account;
use crate::wallet::WalletAccount;

#[cfg(feature = "rpc")]
use crate::rpc::RpcClient;

/// High-level wallet for managing Nano accounts.
///
/// A wallet is created from a seed and can derive multiple accounts.
///
/// # Example
///
/// ```
/// use xno_connect::prelude::*;
/// use xno_connect::wallet::Wallet;
///
/// # fn main() -> xno_connect::error::Result<()> {
/// // Create a wallet from a seed
/// let seed = Seed::from_hex("0000000000000000000000000000000000000000000000000000000000000000")?;
/// let mut wallet = Wallet::from_seed(seed);
///
/// // Get the first account
/// let account = wallet.account(0);
/// println!("Address: {}", account.address());
/// # Ok(())
/// # }
/// ```
pub struct Wallet {
    seed: Seed,
    derived_accounts: Vec<KeyPair>,
}

impl Wallet {
    /// Create a new wallet from a seed.
    pub fn from_seed(seed: Seed) -> Self {
        Wallet {
            seed,
            derived_accounts: Vec::new(),
        }
    }

    /// Create a new wallet with a random seed.
    #[cfg(feature = "std")]
    pub fn new() -> Result<Self> {
        let seed = Seed::random()?;
        Ok(Wallet::from_seed(seed))
    }

    /// Create a wallet from a hex-encoded seed.
    pub fn from_hex_seed(seed_hex: &str) -> Result<Self> {
        let seed = Seed::from_hex(seed_hex)?;
        Ok(Wallet::from_seed(seed))
    }

    /// Get the wallet seed.
    ///
    /// Handle with care - this exposes the secret seed.
    pub fn seed(&self) -> &Seed {
        &self.seed
    }

    /// Get or derive the keypair at the given index.
    fn get_keypair(&mut self, index: u32) -> &KeyPair {
        let index_usize = index as usize;

        // Derive any missing keypairs up to the requested index
        while self.derived_accounts.len() <= index_usize {
            let keypair = self.seed.derive(self.derived_accounts.len() as u32);
            self.derived_accounts.push(keypair);
        }

        &self.derived_accounts[index_usize]
    }

    /// Get a wallet account at the given index.
    pub fn account(&mut self, index: u32) -> WalletAccount {
        let keypair = self.get_keypair(index);
        WalletAccount::new(keypair.clone(), index)
    }

    /// Get the account address at the given index.
    pub fn address(&mut self, index: u32) -> Account {
        self.get_keypair(index).account()
    }

    /// Get multiple account addresses.
    pub fn addresses(&mut self, count: u32) -> Vec<Account> {
        (0..count).map(|i| self.address(i)).collect()
    }

    /// Get the keypair at the given index.
    ///
    /// Useful for signing operations.
    pub fn keypair(&mut self, index: u32) -> &KeyPair {
        self.get_keypair(index)
    }

    // ==================== RPC-dependent methods ====================

    /// Get the balance of an account.
    #[cfg(feature = "rpc")]
    pub async fn balance(
        &mut self,
        index: u32,
        client: &RpcClient,
    ) -> Result<crate::rpc::AccountBalanceResponse> {
        let account = self.address(index);
        client.account_balance(&account).await
    }

    /// Get account info.
    #[cfg(feature = "rpc")]
    pub async fn account_info(
        &mut self,
        index: u32,
        client: &RpcClient,
    ) -> Result<crate::rpc::AccountInfoResponse> {
        let account = self.address(index);
        client.account_info(&account).await
    }

    /// Get account history.
    #[cfg(feature = "rpc")]
    pub async fn history(
        &mut self,
        index: u32,
        count: u64,
        client: &RpcClient,
    ) -> Result<crate::rpc::AccountHistoryResponse> {
        let account = self.address(index);
        client.account_history(&account, count).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SEED: &str = "0000000000000000000000000000000000000000000000000000000000000000";

    #[test]
    fn test_wallet_from_seed() {
        let seed = Seed::from_hex(TEST_SEED).unwrap();
        let wallet = Wallet::from_seed(seed);
        assert!(wallet.derived_accounts.is_empty());
    }

    #[test]
    fn test_wallet_from_hex() {
        let wallet = Wallet::from_hex_seed(TEST_SEED).unwrap();
        assert!(wallet.derived_accounts.is_empty());
    }

    #[test]
    fn test_wallet_address() {
        let mut wallet = Wallet::from_hex_seed(TEST_SEED).unwrap();
        let address = wallet.address(0);

        assert_eq!(
            address.as_str(),
            "nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7"
        );
    }

    #[test]
    fn test_wallet_multiple_addresses() {
        let mut wallet = Wallet::from_hex_seed(TEST_SEED).unwrap();

        let addr0 = wallet.address(0);
        let addr1 = wallet.address(1);
        let addr2 = wallet.address(2);

        // All should be different
        assert_ne!(addr0, addr1);
        assert_ne!(addr1, addr2);
        assert_ne!(addr0, addr2);

        // Should be cached
        assert_eq!(wallet.derived_accounts.len(), 3);
    }

    #[test]
    fn test_wallet_addresses_batch() {
        let mut wallet = Wallet::from_hex_seed(TEST_SEED).unwrap();
        let addresses = wallet.addresses(5);

        assert_eq!(addresses.len(), 5);
        assert_eq!(wallet.derived_accounts.len(), 5);
    }

    #[test]
    fn test_wallet_account() {
        let mut wallet = Wallet::from_hex_seed(TEST_SEED).unwrap();
        let account = wallet.account(0);

        assert_eq!(account.index(), 0);
        assert_eq!(
            account.address().as_str(),
            "nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7"
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_wallet_new_random() {
        let wallet1 = Wallet::new().unwrap();
        let wallet2 = Wallet::new().unwrap();

        // Random wallets should be different
        let mut w1 = wallet1;
        let mut w2 = wallet2;
        assert_ne!(w1.address(0), w2.address(0));
    }
}
