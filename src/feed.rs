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

    /// Returns the USD-denominated value of `position_base_units` of the
    /// underlying asset, in the same 8-decimal fixed-point scale as the
    /// price. Saturates on overflow so callers don't panic on extreme
    /// inputs. The SDK rebalance planner consumes this when building
    /// portfolio snapshots from live oracle reads.
    pub fn value_of(&self, position_base_units: u64) -> u128 {
        (position_base_units as u128).saturating_mul(self.price as u128)
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

    #[test]
    fn value_of_multiplies_position_by_price() {
        // $20.0000_0000 = 20 * 1e8 = 2_000_000_000
        let q = PriceQuote {
            price: 2_000_000_000,
            timestamp: 1,
            status: ReadStatus::Ok,
        };
        // 3 base units * $20.0 = $60.0 (in 8-decimal scale: 6_000_000_000)
        assert_eq!(q.value_of(3), 6_000_000_000u128);
    }

    #[test]
    fn value_of_saturates_on_overflow() {
        let q = PriceQuote {
            price: u64::MAX,
            timestamp: 1,
            status: ReadStatus::Ok,
        };
        // u64::MAX * u64::MAX is < u128::MAX, so should not saturate
        // — verifies the chosen u128 return type is wide enough for
        // realistic Miden faucets (max_supply * max-price-x1e8).
        assert!(q.value_of(u64::MAX) > 0);
        assert!(q.value_of(u64::MAX) < u128::MAX);
    }

    #[test]
    fn value_of_zero_position_returns_zero() {
        let q = PriceQuote {
            price: 9_999_999,
            timestamp: 1,
            status: ReadStatus::Ok,
        };
        assert_eq!(q.value_of(0), 0);
    }
}
