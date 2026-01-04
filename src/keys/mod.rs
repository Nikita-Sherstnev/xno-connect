//! Key management for Nano wallets.
//!
//! This module provides secure key generation, derivation, and signing.

mod derivation;
mod keypair;
mod seed;

pub use derivation::derive_keypair;
pub use keypair::{KeyPair, SecretKey};
pub use seed::Seed;
