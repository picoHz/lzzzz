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
    let acc = match mode {
        CompressionMode::Default => 1,
        CompressionMode::Fast(acc) => acc,
    };
    let state = ExtState::get();
    let len = api::compress_fast_ext_state(&mut state.borrow_mut(), src, dst, acc);
    if len > 0 {
        Ok(len)
    } else {
        Err(LZ4Error::from("Compression failed"))
    }
}

/// Read data from a slice and append compressed data to `Vec<u8>`.
///
/// # Examples
///
/// Compress data into the `Vec<u8>` with the default compression mode:
/// ```
/// use lzzzz::lz4;
///
/// let mut buf = Vec::new();
/// lz4::compress(b"Hello world!", &mut buf, lz4::CompressionMode::Default);
/// ```
///
/// This function doesn't clear the contents of `Vec<u8>`:
/// ```
/// use lzzzz::lz4;
///
/// let header = &b"Compressed data:"[..];
/// let mut buf = Vec::from(header);
/// 
/// lz4::compress(b"Hello world!", &mut buf, lz4::CompressionMode::Default);
/// assert!(buf.starts_with(header));
/// ```
pub fn compress(src: &[u8], dst: &mut Vec<u8>, mode: CompressionMode) -> Result<()> {
    let orig_len = dst.len();
    dst.reserve(max_compressed_size(src.len()));
    #[allow(unsafe_code)]
    unsafe {
        dst.set_len(dst.capacity());
    }
    let result = compress_to_slice(src, &mut dst[orig_len..], mode);
    dst.resize_with(orig_len + *result.as_ref().unwrap_or(&0), Default::default);
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
