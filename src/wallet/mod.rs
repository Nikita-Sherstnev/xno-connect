//! High-level wallet API for Nano.
//!
//! Provides a simple interface for common wallet operations.

mod account;
mod wallet;

pub use account::WalletAccount;
pub use wallet::Wallet;
