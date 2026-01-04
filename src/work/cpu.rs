//! CPU-based work generation.
//!
//! Generates proof of work using CPU threads.

use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};

#[cfg(feature = "work-cpu")]
use rayon::prelude::*;

use crate::error::{Error, Result, WorkError};
use crate::types::{BlockHash, Subtype, Work};
use crate::work::{WorkThreshold, WorkValidator};

/// CPU-based work generator.
///
/// Uses multiple threads (via rayon) to find valid work values.
pub struct CpuWorkGenerator {
    /// Work threshold configuration.
    threshold: WorkThreshold,
    /// Number of threads to use (0 = auto).
    threads: usize,
}

impl CpuWorkGenerator {
    /// Create a new CPU work generator with default settings.
    pub fn new() -> Self {
        CpuWorkGenerator {
            threshold: WorkThreshold::MAINNET,
            threads: 0, // Auto-detect
        }
    }

    /// Set custom work thresholds.
    pub fn with_threshold(mut self, threshold: WorkThreshold) -> Self {
        self.threshold = threshold;
        self
    }

    /// Set the number of threads to use.
    ///
    /// Use 0 for auto-detection (uses all available cores).
    pub fn with_threads(mut self, threads: usize) -> Self {
        self.threads = threads;
        self
    }

    /// Generate work for a hash with the given threshold.
    ///
    /// # Arguments
    /// * `hash` - The block hash (or previous hash for new blocks)
    /// * `threshold` - Minimum difficulty threshold
    /// * `cancelled` - Optional cancellation flag
    ///
    /// # Returns
    /// The work value if found, or an error if cancelled.
    #[cfg(feature = "work-cpu")]
    pub fn generate(
        &self,
        hash: &BlockHash,
        threshold: u64,
        cancelled: Option<&AtomicBool>,
    ) -> Result<Work> {
        let found_flag = Arc::new(AtomicBool::new(false));

        let num_threads = if self.threads == 0 {
            rayon::current_num_threads()
        } else {
            self.threads
        };

        // Divide the search space among threads
        let chunk_size = u64::MAX / num_threads as u64;

        let result: Option<u64> = (0..num_threads).into_par_iter().find_map_any(|i| {
            let start = i as u64 * chunk_size;
            let end = if i == num_threads - 1 {
                u64::MAX
            } else {
                start + chunk_size
            };

            for nonce in start..end {
                // Check cancellation/found flags every 4096 iterations
                if nonce & 0xFFF == 0 {
                    if let Some(cancel) = cancelled {
                        if cancel.load(Ordering::Relaxed) {
                            return None;
                        }
                    }
                    if found_flag.load(Ordering::Relaxed) {
                        return None;
                    }
                }

                let work = Work::new(nonce);
                if WorkValidator::validate(work, hash, threshold) {
                    found_flag.store(true, Ordering::Relaxed);
                    return Some(nonce);
                }
            }

            None
        });

        match result {
            Some(nonce) => Ok(Work::new(nonce)),
            None => {
                if cancelled.map_or(false, |c| c.load(Ordering::Relaxed)) {
                    Err(Error::WorkGeneration(WorkError::Cancelled))
                } else {
                    Err(Error::WorkGeneration(WorkError::MaxIterations))
                }
            }
        }
    }

    /// Generate work for a send/change block.
    #[cfg(feature = "work-cpu")]
    pub fn generate_send(&self, hash: &BlockHash) -> Result<Work> {
        self.generate(hash, self.threshold.send, None)
    }

    /// Generate work for a receive/open block.
    #[cfg(feature = "work-cpu")]
    pub fn generate_receive(&self, hash: &BlockHash) -> Result<Work> {
        self.generate(hash, self.threshold.receive, None)
    }

    /// Generate work for a specific block subtype.
    #[cfg(feature = "work-cpu")]
    pub fn generate_for_subtype(&self, hash: &BlockHash, subtype: Subtype) -> Result<Work> {
        let threshold = self.threshold.for_subtype(subtype);
        self.generate(hash, threshold, None)
    }

    /// Generate work with cancellation support.
    #[cfg(feature = "work-cpu")]
    pub fn generate_cancellable(
        &self,
        hash: &BlockHash,
        subtype: Subtype,
        cancelled: &AtomicBool,
    ) -> Result<Work> {
        let threshold = self.threshold.for_subtype(subtype);
        self.generate(hash, threshold, Some(cancelled))
    }
}

impl Default for CpuWorkGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests are slow as they actually generate work.
    // They use low thresholds for testing.

    const TEST_THRESHOLD: u64 = 0xfffe000000000000; // Lower threshold for faster tests

    #[test]
    #[ignore] // Slow test
    fn test_generate_work_mainnet_difficulty() {
        let generator = CpuWorkGenerator::new();
        let hash =
            BlockHash::from_hex("991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948")
                .unwrap();

        let work = generator
            .generate(&hash, WorkThreshold::MAINNET.send, None)
            .unwrap();

        assert!(WorkValidator::validate(work, &hash, TEST_THRESHOLD));
    }

    #[test]
    #[ignore] // Slow test
    fn test_generate_work() {
        let generator = CpuWorkGenerator::new();
        let hash =
            BlockHash::from_hex("991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948")
                .unwrap();

        let work = generator.generate(&hash, TEST_THRESHOLD, None).unwrap();

        assert!(WorkValidator::validate(work, &hash, TEST_THRESHOLD));
    }

    #[test]
    fn test_generator_creation() {
        let generator = CpuWorkGenerator::new()
            .with_threads(4)
            .with_threshold(WorkThreshold::MAINNET);

        assert_eq!(generator.threads, 4);
        assert_eq!(generator.threshold, WorkThreshold::MAINNET);
    }

    #[test]
    #[ignore] // Slow test
    fn test_cancellation() {
        let generator = CpuWorkGenerator::new();
        let hash =
            BlockHash::from_hex("991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948")
                .unwrap();

        // Set cancelled before starting
        let cancelled = AtomicBool::new(true);

        let result = generator.generate(&hash, u64::MAX, Some(&cancelled));

        assert!(matches!(
            result,
            Err(Error::WorkGeneration(WorkError::Cancelled))
        ));
    }
}
