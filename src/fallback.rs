//! Signed-attestation fallback oracle.
//!
//! When Pragma is unavailable or stale, the adapter reads a Falcon-512
//! signed `(prices, block_number)` tuple from a dedicated note published
//! by a Darwin-operated signer. The pubkey is baked into the adapter at
//! deployment time and is rotatable by the Darwin team.

use crate::feed::PriceQuote;

/// Falcon-512 public key bytes. The exact length depends on
/// miden-crypto's serialisation; we reserve 1793 bytes for the
/// compressed Falcon-512 public key as documented by NIST.
pub type FallbackKey = [u8; 1793];

#[derive(Debug, Clone)]
pub struct SignedAttestation {
    pub block_number: u64,
    pub quotes: Vec<(u64, PriceQuote)>,
    /// Falcon-512 signature over `Poseidon2(block_number || quotes)`.
    pub signature: Vec<u8>,
}
