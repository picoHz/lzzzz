//! LZ4 Block Compressor/Decompressor
mod api;
mod binding;

use crate::{LZ4Error, Result};
use api::ExtState;

/// CompressionMode
///
/// `CompressionMode::Default` is same as `CompressionMode::Fast(1)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompressionMode {
    Default,
    Fast(i32),
}

impl Default for CompressionMode {
    fn default() -> Self {
        Self::Default
    }
}

pub fn max_compressed_size(uncompressed_size: usize) -> usize {
    api::compress_bound(uncompressed_size)
}

pub fn compress_to_slice(src: &[u8], dst: &mut [u8], mode: CompressionMode) -> Result<usize> {
    let state = ExtState::new();
    let acc = match mode {
        CompressionMode::Default => 1,
        CompressionMode::Fast(acc) => acc,
    };
    let len = api::compress_fast_ext_state(&mut state.borrow_mut(), src, dst, acc);
    if len > 0 {
        Ok(len)
    } else {
        Err(LZ4Error::from("Compression failed"))
    }
}

/// Read data from a slice and append a compressed data to `Vec<u8>`.
pub fn compress(src: &[u8], dst: &mut Vec<u8>, mode: CompressionMode) -> Result<()> {
    dst.resize_with(max_compressed_size(src.len()), Default::default);
    let result = compress_to_slice(src, dst, mode);
    dst.resize_with(*result.as_ref().unwrap_or(&0), Default::default);
    result.map(|_| ())
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DecompressionMode<'a> {
    Default,
    Partial,
    Dictionary(&'a [u8]),
}

impl<'a> Default for DecompressionMode<'a> {
    fn default() -> Self {
        Self::Default
    }
}

pub fn decompress(src: &[u8], dst: &mut [u8], mode: DecompressionMode) -> Result<()> {
    todo!();
}
