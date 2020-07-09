#![cfg(feature = "lz4")]
//! Extremely fast compression algorithm

mod binding;
mod block;
mod stream;

pub use block::*;
pub use stream::*;

/// Compression mode specifier
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CompressionMode {
    /// `Default` is same as `Acceleration { factor: 1 }`.
    Default,
    /// Custom acceleration factor.
    Acceleration {
        /// Larger value increases the processing speed in exchange for the
        /// loss of compression ratio.
        factor: i32,
    },
}

impl Default for CompressionMode {
    fn default() -> Self {
        Self::Default
    }
}
