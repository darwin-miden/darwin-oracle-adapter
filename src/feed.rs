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
