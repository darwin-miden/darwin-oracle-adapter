//! Pragma oracle address resolution.

#[derive(Debug, Clone, Copy)]
pub struct AdapterStorageLayout {
    /// Slot holding the current Pragma oracle account ID.
    pub pragma_oracle_id_slot: u8,
    /// Slot holding the maximum acceptable price age, in blocks.
    pub max_price_age_blocks_slot: u8,
    /// Slot holding the public key used to verify fallback attestations.
    pub fallback_pubkey_slot: u8,
}

impl Default for AdapterStorageLayout {
    fn default() -> Self {
        Self {
            pragma_oracle_id_slot: 0,
            max_price_age_blocks_slot: 1,
            fallback_pubkey_slot: 2,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OracleAddress {
    /// Miden account ID prefix (high 64 bits).
    pub prefix: u64,
    /// Miden account ID suffix (low 56 bits or similar — exact encoding
    /// depends on the miden-base version).
    pub suffix: u64,
}
