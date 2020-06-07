//! LZ4 Block Compressor/Decompressor
mod api;
mod binding;

use crate::{LZ4Error, Result};
use api::ExtState;

/// CompressionMode
///
/// `CompressionMode::Default` is same as `CompressionMode::Fast(1)`.
/// # Examples
///
/// Compress data into the `Vec<u8>` with the accelerated compression mode:
/// ```
/// use lzzzz::lz4;
///
/// let mut buf = Vec::new();
/// lz4::compress(
///     b"ex nihilo nihil fit",
///     &mut buf,
///     lz4::CompressionMode::Fast(20),
/// );
/// ```
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

/// Calculate the maximum size of the compressed data from the original size.
pub fn max_compressed_size(uncompressed_size: usize) -> usize {
    api::compress_bound(uncompressed_size)
}

/// Read data from a slice and write compressed data into another slice.
///
/// Ensure that the destination slice have enough capacity.
/// If `dst.len()` is smaller than `max_compressed_size(src.len())`,
/// this function may fail.
///
/// # Examples
///
/// Compress data with the default compression mode:
/// ```
/// use lzzzz::lz4;
///
/// let data = b"Hello world!";
/// let mut buf = [0u8; 2048];
///
/// // The slice should have enough space.
/// assert!(buf.len() >= lz4::max_compressed_size(data.len()));
///
/// lz4::compress_to_slice(data, &mut buf, lz4::CompressionMode::Default);
/// ```
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

/// Read data from a slice and write decompressed data into another slice.
pub fn decompress(src: &[u8], dst: &mut [u8], mode: DecompressionMode) -> Result<()> {
    todo!();
}
