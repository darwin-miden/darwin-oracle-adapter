//! Darwin oracle adapter: a Miden account that sits between the Darwin
//! Protocol Account and Pragma's on-chain oracle.
//!
//! Two responsibilities:
//!
//!   1. **Dynamic Pragma address resolution.** The Pragma oracle account
//!      ID changes between Miden testnet iterations. The adapter holds
//!      the current address in its own storage so that user-facing
//!      Darwin accounts never need to be updated when Pragma rotates.
//!   2. **Signed-attestation fallback.** When Pragma is unreachable or
//!      stale, the adapter falls back to a Darwin-operated signed price
//!      feed whose Falcon-512 public key is baked in at deployment time.
//!
//! Both behaviours are unified behind the WIT interface declared in
//! `wit/oracle.wit`.

pub mod address;
pub mod fallback;
pub mod feed;
pub mod pragma;

#[cfg(feature = "pragma-live")]
pub mod pragma_live;

pub use address::{AdapterStorageLayout, OracleAddress};
pub use fallback::{FallbackKey, SignedAttestation};
pub use feed::{PriceQuote, ReadStatus};
pub use pragma::{
    pair_id_felt, pragma_pair_for_alias, PRAGMA_TESTNET_ORACLE_ID_SNAPSHOT, SUPPORTED_PAIRS,
};

pub const ADAPTER_MASM: &str = include_str!("../asm/adapter.masm");
pub const ORACLE_WIT: &str = include_str!("../wit/oracle.wit");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masm_and_wit_are_bundled() {
        assert!(!ADAPTER_MASM.trim().is_empty());
        assert!(!ORACLE_WIT.trim().is_empty());
        assert!(ORACLE_WIT.contains("interface price-feed"));
    }

    #[test]
    fn read_status_round_trip() {
        for status in [ReadStatus::Ok, ReadStatus::Stale, ReadStatus::FallbackUsed] {
            assert_eq!(status, ReadStatus::from_u8(status.as_u8()).unwrap());
        }
    }
}
