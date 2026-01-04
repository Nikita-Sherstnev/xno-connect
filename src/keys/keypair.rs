//! Ed25519 keypair for signing Nano blocks.
//!
//! Nano uses Ed25519 with Blake2b-512 key expansion (instead of SHA-512).

use alloc::string::String;
use blake2::{Blake2b512, Digest};
use core::fmt;
use curve25519_dalek_ng::{
    constants::ED25519_BASEPOINT_TABLE, edwards::CompressedEdwardsY, scalar::Scalar,
};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::{Error, Result};
use crate::types::{Account, BlockHash, PublicKey, Signature};

/// Secret key (32 bytes).
///
/// The secret key is used to sign blocks. It should never be exposed.
/// Automatically zeroed on drop for security.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecretKey([u8; 32]);

impl SecretKey {
    /// Create from raw bytes.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        SecretKey(bytes)
    }

    /// Get as raw bytes.
    ///
    /// Note: Handle with care - this exposes the secret key.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Create from hex string.
    pub fn from_hex(s: &str) -> Result<Self> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 32 {
            return Err(Error::InvalidPrivateKey);
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(SecretKey(arr))
    }

    /// Convert to hex string.
    ///
    /// Note: Handle with care - this exposes the secret key.
    pub fn to_hex(&self) -> String {
        hex::encode_upper(self.0)
    }
}

impl fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretKey([REDACTED])")
    }
}

/// Clamp scalar bytes for Ed25519.
fn clamp_scalar(bytes: &mut [u8; 32]) {
    bytes[0] &= 248;
    bytes[31] &= 127;
    bytes[31] |= 64;
}

/// Derive the expanded key from a private key using Blake2b-512.
/// Returns (clamped_scalar_bytes, hash_prefix).
fn expand_private_key(private_key: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
    let mut hasher = Blake2b512::new();
    hasher.update(private_key);
    let hash: [u8; 64] = hasher.finalize().into();

    let mut scalar_bytes = [0u8; 32];
    scalar_bytes.copy_from_slice(&hash[0..32]);
    clamp_scalar(&mut scalar_bytes);

    let mut hash_prefix = [0u8; 32];
    hash_prefix.copy_from_slice(&hash[32..64]);

    (scalar_bytes, hash_prefix)
}

/// Ed25519 keypair for Nano.
///
/// Contains both the secret key and derived public key.
/// Used for signing blocks and deriving account addresses.
#[derive(Clone)]
pub struct KeyPair {
    secret_key: SecretKey,
    public_key: PublicKey,
    /// The clamped scalar for signing
    scalar: Scalar,
    /// The hash prefix for deterministic nonce generation
    hash_prefix: [u8; 32],
}

impl KeyPair {
    /// Create a keypair from a private key.
    ///
    /// The public key is derived using Nano's Ed25519 with Blake2b-512 expansion.
    pub fn from_private_key(private_key: [u8; 32]) -> Self {
        let (scalar_bytes, hash_prefix) = expand_private_key(&private_key);

        // Use from_bits to interpret the bytes as a scalar without reduction
        let scalar = Scalar::from_bits(scalar_bytes);

        // Compute public key: A = s * G
        let public_point = &scalar * &ED25519_BASEPOINT_TABLE;
        let public_bytes = public_point.compress().to_bytes();
        let public_key = PublicKey::from_bytes(public_bytes);

        KeyPair {
            secret_key: SecretKey(private_key),
            public_key,
            scalar,
            hash_prefix,
        }
    }

    /// Create a keypair from a secret key.
    pub fn from_secret_key(secret_key: SecretKey) -> Self {
        Self::from_private_key(*secret_key.as_bytes())
    }

    /// Get the secret key.
    pub fn secret_key(&self) -> &SecretKey {
        &self.secret_key
    }

    /// Get the public key.
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Get the account address for this keypair.
    pub fn account(&self) -> Account {
        self.public_key.to_account()
    }

    /// Sign a block hash using Nano's Ed25519 variant.
    pub fn sign(&self, hash: &BlockHash) -> Signature {
        self.sign_message(hash.as_bytes())
    }

    /// Sign arbitrary data using Nano's Ed25519 variant (Blake2b-512 based).
    pub fn sign_message(&self, message: &[u8]) -> Signature {
        // Ed25519 signing:
        // 1. r = H(hash_prefix || message) mod L
        // 2. R = r * G
        // 3. k = H(R || A || message) mod L
        // 4. s = r + k * a (mod L)
        // Signature is (R, s)

        // Step 1: Generate deterministic nonce r
        let mut hasher = Blake2b512::new();
        hasher.update(&self.hash_prefix);
        hasher.update(message);
        let r_hash: [u8; 64] = hasher.finalize().into();
        let r = Scalar::from_bytes_mod_order_wide(&r_hash);

        // Step 2: R = r * G
        let big_r = &r * &ED25519_BASEPOINT_TABLE;
        let big_r_bytes = big_r.compress().to_bytes();

        // Step 3: k = H(R || A || message) mod L
        let mut hasher = Blake2b512::new();
        hasher.update(&big_r_bytes);
        hasher.update(self.public_key.as_bytes());
        hasher.update(message);
        let k_hash: [u8; 64] = hasher.finalize().into();
        let k = Scalar::from_bytes_mod_order_wide(&k_hash);

        // Step 4: s = r + k * a (mod L)
        let s = r + k * self.scalar;

        // Construct signature (R || s)
        let mut sig_bytes = [0u8; 64];
        sig_bytes[..32].copy_from_slice(&big_r_bytes);
        sig_bytes[32..].copy_from_slice(&s.to_bytes());

        Signature::from_bytes(sig_bytes)
    }

    /// Verify a signature.
    pub fn verify(&self, hash: &BlockHash, signature: &Signature) -> bool {
        Self::verify_with_public_key(&self.public_key, hash, signature)
    }

    /// Verify a signature with a public key (static method).
    ///
    /// This uses Nano's Ed25519 verification with Blake2b-512.
    pub fn verify_with_public_key(
        public_key: &PublicKey,
        hash: &BlockHash,
        signature: &Signature,
    ) -> bool {
        Self::verify_message_with_public_key(public_key, hash.as_bytes(), signature)
    }

    /// Verify a signature on arbitrary message data with a public key.
    ///
    /// This uses Nano's Ed25519 verification with Blake2b-512.
    pub fn verify_message_with_public_key(
        public_key: &PublicKey,
        message: &[u8],
        signature: &Signature,
    ) -> bool {
        // Parse R (first 32 bytes of signature)
        let sig_bytes = signature.as_bytes();
        let mut r_bytes = [0u8; 32];
        r_bytes.copy_from_slice(&sig_bytes[..32]);

        let compressed_r = CompressedEdwardsY(r_bytes);
        let r_point = match compressed_r.decompress() {
            Some(p) => p,
            None => return false,
        };

        // Parse s (second 32 bytes of signature)
        let mut s_bytes = [0u8; 32];
        s_bytes.copy_from_slice(&sig_bytes[32..]);

        // s must be reduced mod L - check it's canonical
        let s = Scalar::from_canonical_bytes(s_bytes);
        if s.is_none().into() {
            return false;
        }
        let s = s.unwrap();

        // Parse public key
        let compressed_a = CompressedEdwardsY(*public_key.as_bytes());
        let a_point = match compressed_a.decompress() {
            Some(p) => p,
            None => return false,
        };

        // Compute k = H(R || A || message) mod L using Blake2b-512
        let mut hasher = Blake2b512::new();
        hasher.update(&r_bytes);
        hasher.update(public_key.as_bytes());
        hasher.update(message);
        let k_hash: [u8; 64] = hasher.finalize().into();
        let k = Scalar::from_bytes_mod_order_wide(&k_hash);

        // Verify: s * G == R + k * A
        let lhs = &s * &ED25519_BASEPOINT_TABLE;
        let rhs = r_point + k * a_point;

        lhs == rhs
    }
}

impl fmt::Debug for KeyPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyPair")
            .field("public_key", &self.public_key)
            .field("secret_key", &"[REDACTED]")
            .finish()
    }
}

impl Zeroize for KeyPair {
    fn zeroize(&mut self) {
        self.secret_key.zeroize();
        self.hash_prefix.zeroize();
    }
}

impl Drop for KeyPair {
    fn drop(&mut self) {
        self.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::derive_keypair;

    const ZERO_SEED: [u8; 32] = [0u8; 32];

    #[test]
    fn test_keypair_from_private_key() {
        let keypair = derive_keypair(&ZERO_SEED, 0);

        let expected_pk =
            PublicKey::from_hex("C008B814A7D269A1FA3C6528B19201A24D797912DB9996FF02A1FF356E45552B")
                .unwrap();

        assert_eq!(keypair.public_key(), &expected_pk);
    }

    #[test]
    fn test_keypair_account() {
        let keypair = derive_keypair(&ZERO_SEED, 0);
        let account = keypair.account();

        assert_eq!(
            account.as_str(),
            "nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7"
        );
    }

    #[test]
    fn test_sign_and_verify() {
        let keypair = derive_keypair(&ZERO_SEED, 0);
        let hash =
            BlockHash::from_hex("991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948")
                .unwrap();

        let signature = keypair.sign(&hash);
        assert!(keypair.verify(&hash, &signature));
    }

    #[test]
    fn test_verify_fails_with_wrong_key() {
        let keypair1 = derive_keypair(&ZERO_SEED, 0);
        let keypair2 = derive_keypair(&ZERO_SEED, 1);
        let hash =
            BlockHash::from_hex("991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948")
                .unwrap();

        let signature = keypair1.sign(&hash);
        assert!(!keypair2.verify(&hash, &signature));
    }

    #[test]
    fn test_verify_fails_with_wrong_hash() {
        let keypair = derive_keypair(&ZERO_SEED, 0);
        let hash1 =
            BlockHash::from_hex("991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948")
                .unwrap();
        let hash2 =
            BlockHash::from_hex("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap();

        let signature = keypair.sign(&hash1);
        assert!(!keypair.verify(&hash2, &signature));
    }

    #[test]
    fn test_secret_key_debug_redacted() {
        let sk = SecretKey::from_bytes([0u8; 32]);
        let debug = format!("{:?}", sk);
        assert_eq!(debug, "SecretKey([REDACTED])");
    }

    #[test]
    fn test_keypair_debug_redacted() {
        let keypair = derive_keypair(&ZERO_SEED, 0);
        let debug = format!("{:?}", keypair);
        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains(&keypair.secret_key().to_hex()));
    }

    #[test]
    fn test_secret_key_hex_roundtrip() {
        let original = SecretKey::from_bytes([0xABu8; 32]);
        let hex = original.to_hex();
        let recovered = SecretKey::from_hex(&hex).unwrap();
        assert_eq!(original.as_bytes(), recovered.as_bytes());
    }

    #[test]
    fn test_verify_with_public_key_static() {
        let keypair = derive_keypair(&ZERO_SEED, 0);
        let hash =
            BlockHash::from_hex("991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948")
                .unwrap();

        let signature = keypair.sign(&hash);

        assert!(KeyPair::verify_with_public_key(
            keypair.public_key(),
            &hash,
            &signature
        ));
    }
}
