//! Live Pragma Miden testnet feeds and pair-id resolution.
//!
//! Authoritative source: <https://miden.pragma.build>. The Pragma
//! oracle aggregates whitelisted publishers' price submissions and
//! exposes them via the `agglayer_faucet`-style cross-component API.
//! Addresses change between testnet iterations; the adapter at
//! `asm/adapter.masm` resolves them dynamically at runtime.

/// Snapshot of the Pragma oracle account ID on Miden testnet at the
/// time of M1 implementation work. Not authoritative — the adapter
/// must re-resolve at runtime via its own storage slot. This snapshot
/// exists for tooling and for the test report's appendix.
///
/// Pragma rotates this address between testnet iterations. The
/// canonical user-facing form is the bech32 string at
/// `PRAGMA_TESTNET_ORACLE_BECH32`; the hex form is the
/// `miden-objects::AccountId` decoded from it.
pub const PRAGMA_TESTNET_ORACLE_ID_SNAPSHOT: &str = "0xd0e1384e21a6350029d80128eb5c44";

/// Bech32 form of the current Pragma testnet oracle account, as shown
/// on <https://miden.pragma.build>.
pub const PRAGMA_TESTNET_ORACLE_BECH32: &str = "mtst1argwzwzwyxnr2qpfmqqj366ugsax2t3x";

/// Pragma's internal faucet-id encoding for each pair. Their UI
/// surfaces these as "Faucet ID 1:0" etc. The pair-id felt the MASM
/// adapter passes on the stack is *not* the same as these — see
/// `pair_id_felt`. These constants live here for cross-referencing
/// against Pragma's own dashboard.
pub const PRAGMA_FAUCET_ID_BTC: u32 = 1;
pub const PRAGMA_FAUCET_ID_ETH: u32 = 2;
pub const PRAGMA_FAUCET_ID_WBTC: u32 = 3;
pub const PRAGMA_FAUCET_ID_USDT: u32 = 4;
pub const PRAGMA_FAUCET_ID_DAI: u32 = 5;

/// The Pragma price pairs Darwin reads. Each constituent of every M1
/// basket maps to exactly one pair in this list.
pub const SUPPORTED_PAIRS: &[&str] = &["BTC/USD", "ETH/USD", "WBTC/USD", "USDT/USD", "DAI/USD"];

/// Maps a Darwin asset alias (matching `darwin-baskets` manifests) to
/// its Pragma pair string.
pub fn pragma_pair_for_alias(alias: &str) -> Option<&'static str> {
    match alias {
        "darwin-eth" => Some("ETH/USD"),
        "darwin-wbtc" => Some("WBTC/USD"),
        "darwin-usdt" => Some("USDT/USD"),
        "darwin-dai" => Some("DAI/USD"),
        _ => None,
    }
}

/// Pragma pair identifier as a packed felt. The MASM adapter passes
/// this value on the stack when calling `oracle::get_price`. The
/// production identifier scheme is opaque — we treat it as a stable
/// hash of the pair string for now.
pub fn pair_id_felt(pair: &str) -> u64 {
    // Stable FNV-1a-style hash truncated to fit in a felt. The Pragma
    // oracle uses its own packed-string scheme on-chain; we replicate
    // a deterministic-but-collision-free mapping for off-chain
    // tooling.
    let mut h: u64 = 0xcbf29ce484222325;
    for b in pair.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    // Mask to ~63 bits to stay safely under the Goldilocks modulus.
    h & 0x7fff_ffff_ffff_ffff
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_pairs_match_pragma_testnet() {
        assert!(SUPPORTED_PAIRS.contains(&"ETH/USD"));
        assert!(SUPPORTED_PAIRS.contains(&"WBTC/USD"));
        assert!(SUPPORTED_PAIRS.contains(&"USDT/USD"));
        assert!(SUPPORTED_PAIRS.contains(&"DAI/USD"));
    }

    #[test]
    fn pragma_pair_for_alias_covers_every_m1_constituent() {
        for alias in ["darwin-eth", "darwin-wbtc", "darwin-usdt", "darwin-dai"] {
            assert!(
                pragma_pair_for_alias(alias).is_some(),
                "missing mapping for {alias}"
            );
        }
        assert_eq!(pragma_pair_for_alias("unknown"), None);
    }

    #[test]
    fn pair_id_felt_is_deterministic_and_distinct() {
        let a = pair_id_felt("ETH/USD");
        let b = pair_id_felt("ETH/USD");
        let c = pair_id_felt("WBTC/USD");
        assert_eq!(a, b, "felt id is deterministic");
        assert_ne!(a, c, "different pairs hash to different ids");
    }

    #[test]
    fn pair_id_felt_fits_in_goldilocks_range() {
        for pair in SUPPORTED_PAIRS {
            let id = pair_id_felt(pair);
            // Goldilocks modulus is < 2^64; the safe upper bound for a
            // single-felt encoding is 2^63 (we masked the top bit
            // explicitly).
            assert!(id < (1u64 << 63));
        }
    }
}
