#![cfg(feature = "lz4-hc")]
//! High compression variant of LZ4
//!
//! The `lz4_hc` module doesn't provide decompression functionalities.
//! Use the [`lz4`] module instead.
//!
//! [`lz4`]: ../lz4/index.html

mod binding;
mod block;
mod stream;

pub use block::*;
pub use stream::*;

use std::cmp::Ordering;

/// Compression level specifier
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, Hash)]
pub enum CompressionLevel {
    /// Custom compression level.
    /// Any value between 1 and 12 is valid.
    Custom(i32),
    /// `Min` is same as `Custom(3)`.
    Min,
    /// `Default` is same as `Custom(9)`.
    Default,
    /// `OptMin` is same as `Custom(10)`.
    OptMin,
    /// `Max` is same as `Custom(12)`.
    Max,
}

impl PartialOrd for CompressionLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.as_i32().cmp(&other.as_i32()))
    }
}

impl Default for CompressionLevel {
    fn default() -> Self {
        Self::Default
    }
}

impl CompressionLevel {
    pub(crate) fn as_i32(self) -> i32 {
        match self {
            Self::Custom(level) => level,
            Self::Min => 3,
            Self::Default => 9,
            Self::OptMin => 10,
            Self::Max => 12,
        }
    }
}

/// Decompression speed flag
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FavorDecSpeed {
    /// Default value
    Disabled,
    Enabled,
}

impl Default for FavorDecSpeed {
    fn default() -> Self {
        Self::Disabled
    }
}
