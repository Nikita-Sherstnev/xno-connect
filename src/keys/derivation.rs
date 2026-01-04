//! Key derivation for Nano.
//!
//! Nano uses a simple Blake2b-based key derivation scheme:
//! private_key = blake2b(seed || index)
//!
//! The index is encoded as a 32-bit big-endian integer.

use blake2::digest::consts::U32;
use blake2::{Blake2b, Digest};

use crate::keys::KeyPair;

/// Derive a keypair from a seed at the given index.
///
/// This implements Nano's key derivation:
/// `private_key = blake2b-256(seed || index_be32)`
///
/// # Arguments
/// * `seed` - The 32-byte master seed
/// * `index` - The account index (0 for first account)
///
/// # Returns
/// A keypair containing the derived private and public keys.
pub fn derive_keypair(seed: &[u8; 32], index: u32) -> KeyPair {
    let mut hasher = Blake2b::<U32>::new();
    hasher.update(seed);
    hasher.update(&index.to_be_bytes());

    let private_key: [u8; 32] = hasher.finalize().into();
    KeyPair::from_private_key(private_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PublicKey;

    // Test vectors from Nano documentation
    // Seed: 0000...0000 (all zeros)
    // Index 0 -> Known public key
    const ZERO_SEED: [u8; 32] = [0u8; 32];

    #[test]
    fn test_derive_index_0() {
        let keypair = derive_keypair(&ZERO_SEED, 0);
        let expected_pk =
            PublicKey::from_hex("C008B814A7D269A1FA3C6528B19201A24D797912DB9996FF02A1FF356E45552B")
                .unwrap();

        assert_eq!(keypair.public_key(), &expected_pk);
    }

    #[test]
    fn test_derive_index_1() {
        let keypair = derive_keypair(&ZERO_SEED, 1);

        let expected_pk =
            PublicKey::from_hex("E30D22B7935BCC25412FC07427391AB4C98A4AD68BAA733300D23D82C9D20AD3")
                .unwrap();

        assert_eq!(keypair.public_key(), &expected_pk);
    }

    #[test]
    fn test_different_indices_produce_different_keys() {
        let kp0 = derive_keypair(&ZERO_SEED, 0);
        let kp1 = derive_keypair(&ZERO_SEED, 1);
        let kp2 = derive_keypair(&ZERO_SEED, 2);

        assert_ne!(kp0.public_key(), kp1.public_key());
        assert_ne!(kp1.public_key(), kp2.public_key());
        assert_ne!(kp0.public_key(), kp2.public_key());
    }

    #[test]
    fn test_same_index_produces_same_key() {
        let kp1 = derive_keypair(&ZERO_SEED, 42);
        let kp2 = derive_keypair(&ZERO_SEED, 42);

        assert_eq!(kp1.public_key(), kp2.public_key());
    }

    #[test]
    fn test_different_seeds_produce_different_keys() {
        let seed1 = [0u8; 32];
        let seed2 = [1u8; 32];

        let kp1 = derive_keypair(&seed1, 0);
        let kp2 = derive_keypair(&seed2, 0);

        assert_ne!(kp1.public_key(), kp2.public_key());
    }
}
