//! Live Pragma oracle integration.
//!
//! Computes the MAST roots of `get_median` / `get_entry` on the real
//! Pragma oracle by re-running their build pipeline locally (the same
//! `pm-accounts` crate that Pragma uses to deploy). Discovers the set
//! of publisher accounts the deployed oracle references, and builds
//! the `ForeignAccount` map miden-client needs so a Darwin transaction
//! can call `call.<get_median_mast_root>` on the Pragma oracle.
//!
//! Reference: <https://github.com/astraly-labs/pragma-miden>
//!
//! This module is gated behind the `pragma-live` Cargo feature.

use std::collections::BTreeMap;

use anyhow::{anyhow, Context, Result};
use miden_client::Client;
use miden_client::account::AccountId;
use miden_client::keystore::FilesystemKeyStore;
use miden_client::rpc::domain::account::AccountStorageRequirements;
use miden_client::transaction::ForeignAccount;
use miden_protocol::account::{StorageMapKey, StorageSlotName};

/// The Pragma oracle account on Miden testnet, observed at
/// <https://miden.pragma.build>. Pragma rotates this between testnet
/// iterations; the production adapter (in `asm/adapter.masm`) resolves
/// it dynamically from its own storage slot.
pub const PRAGMA_TESTNET_ORACLE_HEX: &str = "0xd0e1384e21a6350029d80128eb5c44";

/// Pragma's publisher contract on Miden testnet, as published at
/// <https://miden.pragma.build>. Bech32 form is the canonical user-
/// facing identifier; we decode to AccountId via
/// `miden_protocol::account::AccountId::from_bech32` at runtime.
pub const PRAGMA_TESTNET_PUBLISHER_BECH32: &str = "mtst1aqhn4fnd8x8duqr9y4ku23qqwyzdndw3";

/// Storage slot names defined by Pragma in `oracle/mod.rs`
/// (`oracle_storage_slots`).
pub const ORACLE_NEXT_PUBLISHER_INDEX_SLOT: &str = "pragma::oracle::next_publisher_index";
pub const ORACLE_PUBLISHERS_MAP_SLOT: &str = "pragma::oracle::publishers";

/// Storage slot name defined by Pragma in `publisher/mod.rs`
/// (`PublisherAccountBuilder::new`).
pub const PUBLISHER_ENTRIES_SLOT: &str = "pragma::publisher::entries";

/// Returns the MAST root of Pragma's oracle `get_median` procedure as
/// a `0x`-prefixed 64-char hex string suitable for `call.0x…` in a
/// transaction script.
///
/// Internally this re-runs Pragma's build pipeline:
///   1. compile `publisher.masm` to compute `get_entry_procedure_hash`
///   2. substitute that hash into the `{GET_ENTRY_HASH}` placeholder
///      in `oracle.masm`
///   3. compile the patched oracle module
///   4. look up the digest of the `get_median` export.
///
/// Because the deployed oracle on Miden testnet was assembled by
/// Pragma using the same template + same publisher source, the hash
/// returned here matches the on-chain MAST root for `get_median`.
pub fn pragma_get_median_mast_root_hex() -> String {
    // pm-accounts' helper returns a dot-separated `f0.f1.f2.f3` of
    // four felt u64s. We convert to the canonical 32-byte big-endian
    // hex string the transaction-script call site expects.
    let dotted = pm_accounts::oracle::get_median_procedure_hash();
    dotted_felts_to_hex(&dotted).expect("pm-accounts returns four well-formed u64s")
}

/// Same as `pragma_get_median_mast_root_hex` but for the publisher's
/// `get_entry` procedure — useful for sanity checks and for cases
/// where Darwin wants to call a publisher directly.
pub fn publisher_get_entry_mast_root_hex() -> String {
    // pm-accounts returns publisher's get_entry hash in *reversed* felt
    // order vs oracle's get_median (see publisher/mod.rs:50). Keep the
    // same order they emit so the value matches what the oracle has
    // patched into `{GET_ENTRY_HASH}`.
    let dotted = pm_accounts::publisher::get_entry_procedure_hash();
    dotted_felts_to_hex(&dotted).expect("pm-accounts returns four well-formed u64s")
}

/// Convert pm-accounts' `"<f0>.<f1>.<f2>.<f3>"` digest representation
/// to a `0x`-prefixed 64-char hex digest (8 bytes per felt,
/// little-endian — matching how miden-vm pushes word digests on the
/// stack).
fn dotted_felts_to_hex(dotted: &str) -> Result<String> {
    let parts: Vec<&str> = dotted.split('.').collect();
    if parts.len() != 4 {
        return Err(anyhow!(
            "expected 4 dot-separated felts, got {} from {dotted:?}",
            parts.len()
        ));
    }
    let mut bytes = Vec::with_capacity(32);
    for p in parts {
        let f: u64 = p.parse().with_context(|| format!("parse felt {p:?}"))?;
        bytes.extend_from_slice(&f.to_le_bytes());
    }
    Ok(format!("0x{}", hex::encode(bytes)))
}

/// Read the oracle's publisher list off-chain via miden-client.
///
/// The deployed oracle stores at `pragma::oracle::next_publisher_index`
/// the number of publishers `n`, and at `pragma::oracle::publishers`
/// a map from `Word([i,0,0,0])` → publisher AccountId word, for
/// `i ∈ [PUBLISHERS_STORAGE_SLOT_START .. PUBLISHERS_STORAGE_SLOT_START + n)`.
///
/// In practice we don't need to walk the on-chain map: pre-registering
/// each publisher as a foreign account requires their `AccountId`, and
/// the canonical way to get those is from Pragma's public list (their
/// site shows them). We expose the structural helper here for the
/// production code path; for the M1 demo we use the snapshot constant
/// `PRAGMA_TESTNET_PUBLISHER_HEX`.
pub async fn discover_publishers(
    _client: &mut Client<FilesystemKeyStore>,
    _oracle_id: AccountId,
) -> Result<Vec<AccountId>> {
    // The map read path requires walking `get_map_item` which in
    // miden-client 0.14 is not directly exposed through the public
    // account API for cross-account reads (it requires a tx-script
    // round-trip). For the M1 demo we use the publisher snapshot
    // published on <https://miden.pragma.build>.
    //
    // Production swap: once miden-client exposes
    // `client.get_account_storage_map_item(oracle_id, slot, key)`
    // (planned 0.15), this becomes a tight loop.
    let (_, snap) = AccountId::from_bech32(PRAGMA_TESTNET_PUBLISHER_BECH32)
        .map_err(|e| anyhow!("decode publisher bech32: {e}"))?;
    Ok(vec![snap])
}

/// Build the `ForeignAccount` map miden-client needs at
/// `execute_program` / `submit_transaction` time so the kernel can
/// resolve cross-component calls into the Pragma oracle and each
/// publisher account.
///
/// The map is keyed by account id, and includes:
///   * the oracle itself, with storage requirements for the
///     publishers map (so the oracle can read its publishers list);
///   * each publisher account, with storage requirements for the
///     `pragma::publisher::entries` map keyed on the queried pair
///     word — so the kernel can resolve `publisher::get_entry`'s
///     `get_map_item` lookup.
pub async fn build_foreign_accounts(
    client: &mut Client<FilesystemKeyStore>,
    oracle_id: AccountId,
    publishers: &[AccountId],
    pair_word: [u64; 4],
) -> Result<BTreeMap<AccountId, ForeignAccount>> {
    let mut map = BTreeMap::new();

    // Oracle needs the publishers map slot accessible.
    let publishers_slot_name = StorageSlotName::new(ORACLE_PUBLISHERS_MAP_SLOT)
        .map_err(|e| anyhow!("oracle publishers slot name: {e}"))?;
    let oracle_req = AccountStorageRequirements::new([(
        publishers_slot_name,
        &[][..],
    )]);
    let oracle_fa = ForeignAccount::public(oracle_id, oracle_req)
        .map_err(|e| anyhow!("build oracle foreign account: {e}"))?;
    map.insert(oracle_fa.account_id(), oracle_fa);

    // Each publisher needs its entries map slot accessible, keyed on
    // the pair word.
    let entries_slot_name = StorageSlotName::new(PUBLISHER_ENTRIES_SLOT)
        .map_err(|e| anyhow!("publisher entries slot name: {e}"))?;
    let pair_key = StorageMapKey::new(pair_word.map(miden_protocol::Felt::new).into());
    for &pub_id in publishers {
        client
            .import_account_by_id(pub_id)
            .await
            .with_context(|| format!("import publisher {pub_id}"))?;
        let req = AccountStorageRequirements::new([(
            entries_slot_name.clone(),
            &[pair_key.clone()][..],
        )]);
        let fa = ForeignAccount::public(pub_id, req)
            .map_err(|e| anyhow!("build publisher foreign account: {e}"))?;
        map.insert(fa.account_id(), fa);
    }

    Ok(map)
}

/// Pragma's per-pair faucet id (prefix, suffix). Verbatim from
/// <https://miden.pragma.build>: each pair is identified by a
/// `prefix:suffix` integer pair (BTC/USD = 1:0, ETH/USD = 2:0, etc.).
pub fn faucet_id_for_pair(pair: &str) -> Option<(u64, u64)> {
    match pair {
        "BTC/USD" => Some((1, 0)),
        "ETH/USD" => Some((2, 0)),
        "WBTC/USD" => Some((3, 0)),
        "USDT/USD" => Some((4, 0)),
        "DAI/USD" => Some((5, 0)),
        _ => None,
    }
}

/// Encode a Pragma price pair into the 4-felt word the publisher's
/// `pragma::publisher::entries` storage map is keyed by.
///
/// Format taken verbatim from Pragma's `pm-oracle-cli median` source
/// (`crates/cli/oracle/src/commands/median.rs`):
///
/// ```text
/// let faucet_id_word: Word = [
///     Felt::new(0),
///     Felt::new(0),
///     Felt::new(suffix),
///     Felt::new(prefix),
/// ].into();
/// ```
///
/// i.e. `[0, 0, suffix, prefix]` — the two zeros are left-padding and
/// the integer faucet id occupies the lower two felts.
pub fn pair_word(pair: &str) -> Option<[u64; 4]> {
    let (prefix, suffix) = faucet_id_for_pair(pair)?;
    Some([0, 0, suffix, prefix])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dotted_felts_round_trip_to_hex_is_64_chars() {
        // 4 felts × 8 bytes = 32 bytes = 64 hex chars + "0x" prefix
        let dotted = "1.2.3.4";
        let hex = dotted_felts_to_hex(dotted).unwrap();
        assert!(hex.starts_with("0x"));
        assert_eq!(hex.len(), 2 + 64);
    }

    #[test]
    fn dotted_felts_rejects_wrong_arity() {
        assert!(dotted_felts_to_hex("1.2.3").is_err());
        assert!(dotted_felts_to_hex("1.2.3.4.5").is_err());
    }

    #[test]
    fn pragma_get_median_mast_root_is_well_formed() {
        // Just exercises the compile-locally + extract-digest path.
        // The actual value is whatever pm-accounts computes — we don't
        // hardcode it here because it changes whenever Pragma rebuilds.
        let hex = pragma_get_median_mast_root_hex();
        assert!(hex.starts_with("0x"));
        assert_eq!(hex.len(), 2 + 64);
        println!("pragma::oracle::get_median MAST root: {hex}");
    }

    #[test]
    fn publisher_get_entry_mast_root_is_well_formed() {
        let hex = publisher_get_entry_mast_root_hex();
        assert!(hex.starts_with("0x"));
        assert_eq!(hex.len(), 2 + 64);
        println!("pragma::publisher::get_entry MAST root: {hex}");
    }

    #[test]
    fn pair_word_matches_pragma_cli_layout() {
        // ETH/USD → faucet 2:0 → [0, 0, 0, 2]
        assert_eq!(pair_word("ETH/USD"), Some([0, 0, 0, 2]));
        // BTC/USD → faucet 1:0 → [0, 0, 0, 1]
        assert_eq!(pair_word("BTC/USD"), Some([0, 0, 0, 1]));
        // DAI/USD → faucet 5:0 → [0, 0, 0, 5]
        assert_eq!(pair_word("DAI/USD"), Some([0, 0, 0, 5]));
        assert_eq!(pair_word("UNK/USD"), None);
    }

    #[test]
    fn faucet_id_covers_every_supported_pair() {
        for pair in super::super::pragma::SUPPORTED_PAIRS {
            assert!(
                faucet_id_for_pair(pair).is_some(),
                "missing faucet id for {pair}"
            );
        }
    }
}
