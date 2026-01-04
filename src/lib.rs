//! # XNO-connect
//!
//! A Rust library for Nano cryptocurrency node communication via RPC and WebSocket APIs. Includes
//! WASM support and local work generation.
//!
//! ## Features
//!
//! - **Wallet Management**: Create wallets, derive accounts, manage keys securely
//! - **Block Operations**: Create, sign, and hash state blocks
//! - **RPC Client**: Communicate with Nano nodes via JSON-RPC
//! - **WebSocket Client**: Subscribe to real-time confirmations and votes
//! - **Work Generation**: Generate PoW locally or via external work servers
//! - **WASM Support**: Optional WebAssembly support for browser environments
//!
//! ## Example
//!
//! ```rust,no_run
//! use xno_connect::prelude::*;
//!
//! // Create a new wallet from a random seed
//! let seed = Seed::random().expect("Failed to generate seed");
//! let keypair = seed.derive(0);
//!
//! // Get account address
//! let account = keypair.account();
//! println!("Account: {}", account);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]

extern crate alloc;

pub mod blocks;
pub mod error;
pub mod keys;
pub mod types;
pub mod work;

#[cfg(any(feature = "rpc", feature = "wasm-rpc"))]
pub mod rpc;

#[cfg(any(feature = "websocket", feature = "wasm-websocket"))]
pub mod websocket;

pub mod wallet;

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::blocks::{BlockBuilder, BlockHasher};
    pub use crate::error::{Error, Result};
    pub use crate::keys::{KeyPair, SecretKey, Seed};
    pub use crate::types::{
        Account, Amount, BlockHash, PublicKey, Raw, Signature, StateBlock, Subtype, Work,
    };
    pub use crate::work::{WorkThreshold, WorkValidator};

    #[cfg(any(feature = "rpc", feature = "wasm-rpc"))]
    pub use crate::rpc::RpcClient;

    #[cfg(any(feature = "websocket", feature = "wasm-websocket"))]
    pub use crate::websocket::WebSocketClient;

    pub use crate::wallet::Wallet;
}

pub use error::{Error, Result};

/// Nano network constants.
pub mod constants {
    /// Nano's base32 alphabet for account encoding.
    pub const BASE32_ALPHABET: &[u8; 32] = b"13456789abcdefghijkmnopqrstuwxyz";

    /// Account prefix for mainnet.
    pub const ACCOUNT_PREFIX_NANO: &str = "nano_";

    /// Alternative account prefix.
    pub const ACCOUNT_PREFIX_XNO: &str = "xno_";

    /// Work difficulty threshold for send/change blocks (mainnet).
    pub const WORK_THRESHOLD_SEND: u64 = 0xfffffff800000000;

    /// Work difficulty threshold for receive blocks (mainnet).
    pub const WORK_THRESHOLD_RECEIVE: u64 = 0xfffffe0000000000;

    /// Epoch v2 work threshold for send blocks.
    pub const WORK_THRESHOLD_EPOCH_2_SEND: u64 = 0xfffffff800000000;

    /// Epoch v2 work threshold for receive blocks.
    pub const WORK_THRESHOLD_EPOCH_2_RECEIVE: u64 = 0xfffffe0000000000;

    /// Maximum raw supply (2^128 - 1).
    pub const MAX_SUPPLY_RAW: u128 = 340282366920938463463374607431768211455;

    /// 1 Nano (XNO) in raw units (10^30 raw).
    pub const NANO_IN_RAW: u128 = 1_000_000_000_000_000_000_000_000_000_000;

    /// State block preamble for hashing.
    pub const STATE_BLOCK_PREAMBLE: [u8; 32] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 6,
    ];

    /// Zero hash (32 bytes of zeros).
    pub const ZERO_HASH: [u8; 32] = [0u8; 32];

    /// Zero public key (burn address).
    pub const ZERO_PUBLIC_KEY: [u8; 32] = [0u8; 32];
}
