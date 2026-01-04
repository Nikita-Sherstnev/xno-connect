//! WebSocket client for real-time Nano node communication.
//!
//! Provides subscription-based updates for confirmations, votes, and more.

mod client;
mod messages;
mod subscription;

pub use client::WebSocketClient;
pub use messages::*;
pub use subscription::*;
