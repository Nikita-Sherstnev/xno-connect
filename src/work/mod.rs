//! Proof of Work generation and validation.
//!
//! Nano uses a proof of work system to prevent spam. Work must be computed
//! for each block before it can be processed by the network.
//! For remote work generation use RPC request.

mod validate;

#[cfg(feature = "work-cpu")]
mod cpu;

pub use validate::{WorkThreshold, WorkValidator};

#[cfg(feature = "work-cpu")]
pub use cpu::CpuWorkGenerator;
