//! RPC client for Nano node communication.
//!
//! Provides an asynchronous client for interacting with Nano nodes via JSON-RPC.
//! Works on both native and WASM platforms.
//!
//! # Example
//!
//! ```no_run
//! use xno_connect::rpc::RpcClient;
//!
//! # async fn example() -> xno_connect::error::Result<()> {
//! let client = RpcClient::new("http://localhost:7076");
//! let account = "nano_1abc...".parse()?;
//! let balance = client.account_balance(&account).await?;
//! println!("Balance: {}", balance.balance);
//! # Ok(())
//! # }
//! ```

mod client;
mod requests;
mod responses;

pub use client::RpcClient;
pub use requests::*;
pub use responses::*;
