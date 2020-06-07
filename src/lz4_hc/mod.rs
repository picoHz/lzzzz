//! LZ4_HC Block Compressor
//!
//! The `lz4_hc` module doesn't provide decompression functionalities.
//! Use the `lz4` module instead.
mod api;
mod binding;

use crate::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompressionMode {
    Default,
    DestSize,
}

impl Default for CompressionMode {
    fn default() -> Self {
        Self::Default
    }
}

/// Compression level.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CompressionLevel {
    Custom(i32),
    Min,
    Default,
    OptMin,
    Max,
}

impl CompressionLevel {}

pub fn max_compressed_size(uncompressed_size: usize) -> usize {
    0
}

pub fn compress_to_slice(
    src: &[u8],
    dst: &mut [u8],
    mode: CompressionMode,
    compression_level: CompressionLevel,
) -> Result<usize> {
    todo!();
}

/// Read data from a slice and append a compressed data to `Vec<u8>`.
pub fn compress(
    src: &[u8],
    dst: &mut Vec<u8>,
    mode: CompressionMode,
    compression_level: CompressionLevel,
) -> Result<()> {
    todo!();
}
