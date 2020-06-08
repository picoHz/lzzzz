//! LZ4_HC Block Compressor
//!
//! The `lz4_hc` module doesn't provide decompression functionalities.
//! Use the `lz4` module instead.
mod api;

use crate::Result;
use api::ExtState;

/// Compression mode specifier
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompressionMode {
    Default,
    DestSize { uncompressed_size: usize },
}

impl Default for CompressionMode {
    fn default() -> Self {
        Self::Default
    }
}

/// Compression level
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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

impl CompressionLevel {}

/// Calculate the maximum size of the compressed data from the original size.
pub fn max_compressed_size(uncompressed_size: usize) -> usize {
    0
}

/// Read data from a slice and write compressed data into another slice.
pub fn compress_to_slice(
    src: &[u8],
    dst: &mut [u8],
    mode: CompressionMode,
    compression_level: CompressionLevel,
) -> Result<usize> {
    todo!();
}

/// Read data from a slice and append compressed data to `Vec<u8>`.
pub fn compress(
    src: &[u8],
    dst: &mut Vec<u8>,
    mode: CompressionMode,
    compression_level: CompressionLevel,
) -> Result<()> {
    todo!();
}

thread_local!(static EXT_STATE: ExtState = ExtState::new());
