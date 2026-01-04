//! Work validation for Nano blocks.
//!
//! Work is validated by computing:
//! difficulty = blake2b(work_bytes || hash)[0..8] as u64 (little-endian)
//!
//! The difficulty must be greater than or equal to the threshold.

use blake2::digest::consts::U8;
use blake2::{Blake2b, Digest};

use crate::constants::{WORK_THRESHOLD_RECEIVE, WORK_THRESHOLD_SEND};
use crate::types::{BlockHash, Subtype, Work};

/// Work difficulty thresholds for different block types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkThreshold {
    /// Threshold for send/change blocks.
    pub send: u64,
    /// Threshold for receive/open blocks.
    pub receive: u64,
}

impl WorkThreshold {
    /// Mainnet thresholds (epoch v2).
    pub const MAINNET: WorkThreshold = WorkThreshold {
        send: WORK_THRESHOLD_SEND,
        receive: WORK_THRESHOLD_RECEIVE,
    };

    /// Get the threshold for a specific block subtype.
    pub fn for_subtype(&self, subtype: Subtype) -> u64 {
        match subtype {
            Subtype::Send | Subtype::Change | Subtype::Epoch => self.send,
            Subtype::Receive | Subtype::Open => self.receive,
        }
    }

    /// Get the threshold for a send/change block.
    pub fn for_send(&self) -> u64 {
        self.send
    }

    /// Get the threshold for a receive/open block.
    pub fn for_receive(&self) -> u64 {
        self.receive
    }
}

impl Default for WorkThreshold {
    fn default() -> Self {
        Self::MAINNET
    }
}

/// Work validator for checking proof of work.
pub struct WorkValidator;

impl WorkValidator {
    /// Calculate the difficulty of a work value for a given hash.
    ///
    /// Returns the 64-bit difficulty value. Higher is better.
    pub fn difficulty(work: Work, hash: &BlockHash) -> u64 {
        let mut hasher = Blake2b::<U8>::new();

        // Work is hashed as little-endian bytes
        hasher.update(&work.to_le_bytes());
        hasher.update(hash.as_bytes());

        let result: [u8; 8] = hasher.finalize().into();

        // Result is interpreted as little-endian u64
        u64::from_le_bytes(result)
    }

    /// Validate work against a threshold.
    ///
    /// Returns true if the work difficulty is at least the threshold.
    pub fn validate(work: Work, hash: &BlockHash, threshold: u64) -> bool {
        Self::difficulty(work, hash) >= threshold
    }

    /// Validate work for a send/change block.
    pub fn validate_send(work: Work, hash: &BlockHash) -> bool {
        Self::validate(work, hash, WORK_THRESHOLD_SEND)
    }

    /// Validate work for a receive/open block.
    pub fn validate_receive(work: Work, hash: &BlockHash) -> bool {
        Self::validate(work, hash, WORK_THRESHOLD_RECEIVE)
    }

    /// Validate work for a specific block subtype.
    pub fn validate_for_subtype(work: Work, hash: &BlockHash, subtype: Subtype) -> bool {
        let threshold = WorkThreshold::MAINNET.for_subtype(subtype);
        Self::validate(work, hash, threshold)
    }

    /// Get the multiplier of the work difficulty relative to the threshold.
    ///
    /// Returns a value >= 1.0 if valid, < 1.0 if invalid.
    pub fn multiplier(work: Work, hash: &BlockHash, threshold: u64) -> f64 {
        let difficulty = Self::difficulty(work, hash);
        let base = u64::MAX - threshold;
        let actual = u64::MAX - difficulty;

        if actual == 0 {
            f64::MAX
        } else {
            base as f64 / actual as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_calculation() {
        // Test with known values
        let work = Work::from_hex("7202df8a7c380578").unwrap();
        let hash =
            BlockHash::from_hex("991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948")
                .unwrap();

        let difficulty = WorkValidator::difficulty(work, &hash);

        // Should produce some non-zero difficulty
        assert!(difficulty > 0);
    }

    #[test]
    fn test_threshold_for_subtype() {
        let threshold = WorkThreshold::MAINNET;

        assert_eq!(threshold.for_subtype(Subtype::Send), WORK_THRESHOLD_SEND);
        assert_eq!(threshold.for_subtype(Subtype::Change), WORK_THRESHOLD_SEND);
        assert_eq!(
            threshold.for_subtype(Subtype::Receive),
            WORK_THRESHOLD_RECEIVE
        );
        assert_eq!(threshold.for_subtype(Subtype::Open), WORK_THRESHOLD_RECEIVE);
    }

    #[test]
    fn test_receive_threshold_lower_than_send() {
        // Receive threshold should be lower (easier) than send
        assert!(WORK_THRESHOLD_RECEIVE < WORK_THRESHOLD_SEND);
    }

    #[test]
    fn test_validate_zero_work_fails() {
        let hash =
            BlockHash::from_hex("991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948")
                .unwrap();

        // Zero work should almost always fail
        assert!(!WorkValidator::validate_send(Work::ZERO, &hash));
    }

    #[test]
    fn test_work_threshold_constants() {
        // Verify the thresholds are reasonable
        assert!(WORK_THRESHOLD_SEND > 0);
        assert!(WORK_THRESHOLD_RECEIVE > 0);

        // Send should require more work than receive
        assert!(WORK_THRESHOLD_SEND > WORK_THRESHOLD_RECEIVE);
    }

    #[test]
    fn test_multiplier_calculation() {
        let work = Work::from_hex("7202df8a7c380578").unwrap();
        let hash =
            BlockHash::from_hex("991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948")
                .unwrap();

        let multiplier = WorkValidator::multiplier(work, &hash, WORK_THRESHOLD_RECEIVE);

        // Should be a positive number
        assert!(multiplier > 0.0);
    }

    #[test]
    fn test_different_hashes_produce_different_difficulties() {
        let work = Work::from_hex("7202df8a7c380578").unwrap();

        let hash1 =
            BlockHash::from_hex("991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948")
                .unwrap();

        let hash2 =
            BlockHash::from_hex("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap();

        let diff1 = WorkValidator::difficulty(work, &hash1);
        let diff2 = WorkValidator::difficulty(work, &hash2);

        assert_ne!(diff1, diff2);
    }
}
