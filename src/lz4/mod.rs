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

/// Decompression mode specifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DecompressionMode<'a> {
    /// Decompress the whole data.
    ///
    /// The destination slice must have the exact size of the uncompressed data.
    Default,
    /// Decompress the partial data.
    ///
    /// The destination slice can have smaller size of the uncompressed data.
    Partial {
        /// The value must be the exact size of the uncompressed data.
        original_size: usize,
    },
    Dictionary {
        data: &'a [u8],
    },
}

impl<'a> Default for DecompressionMode<'a> {
    fn default() -> Self {
        Self::Default
    }
}
