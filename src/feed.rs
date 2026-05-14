//! Rust mirror of the WIT `price-quote` and `read-status` types.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadStatus {
    Ok = 0,
    Stale = 1,
    FallbackUsed = 2,
}

impl ReadStatus {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(b: u8) -> Option<Self> {
        match b {
            0 => Some(ReadStatus::Ok),
            1 => Some(ReadStatus::Stale),
            2 => Some(ReadStatus::FallbackUsed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PriceQuote {
    /// USD price with 8 decimals of precision.
    pub price: u64,
    /// Miden block number at which the price was last updated.
    pub timestamp: u64,
    pub status: ReadStatus,
}

impl PriceQuote {
    /// Spec §8.5: the adapter treats prices as stale when their
    /// timestamp lags the current block by more than this many blocks.
    /// 10 blocks ≈ 10–20 s on Miden testnet, matching Pragma's
    /// publisher cadence.
    pub const MAX_PRICE_AGE_BLOCKS: u64 = 10;

    /// Returns true if this quote is fresh relative to `current_block`.
    pub fn is_fresh(&self, current_block: u64) -> bool {
        current_block.saturating_sub(self.timestamp) <= Self::MAX_PRICE_AGE_BLOCKS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quote_is_fresh_within_max_age() {
        let q = PriceQuote {
            price: 100,
            timestamp: 100,
            status: ReadStatus::Ok,
        };
        assert!(q.is_fresh(100));
        assert!(q.is_fresh(105));
        assert!(q.is_fresh(110));
        assert!(!q.is_fresh(111));
    }

    #[test]
    fn fresh_check_does_not_underflow() {
        let q = PriceQuote {
            price: 0,
            timestamp: 100,
            status: ReadStatus::Ok,
        };
        // current_block < timestamp shouldn't panic.
        assert!(q.is_fresh(50));
    }
}
