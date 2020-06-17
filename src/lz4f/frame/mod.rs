//! LZ4 Frame Compressor/Decompressor

mod api;

use super::stream::DecompressionContext;
use crate::lz4f::Preferences;
use crate::{Error, Report, Result};
use std::cell::RefCell;

/// Calculate the maximum size of the compressed data from the original size.
pub fn max_compressed_size(uncompressed_size: usize, prefs: &Preferences) -> usize {
    api::compress_bound(uncompressed_size, prefs)
}

/// Read data from a slice and write compressed data into another slice.
///
/// Ensure that the destination slice have enough capacity.
/// If `dst.len()` is smaller than `lz4::max_compressed_size(src.len())`,
/// this function may fail.
///
/// # Examples
///
/// Compress data with the default compression mode:
/// ```
/// use lzzzz::lz4f;
///
/// let data = b"As soon as they had strength, they arose, joined hands again, and went on.";
/// let mut buf = [0u8; 131_072];
/// let prefs = lz4f::Preferences::default();
///
/// // The slice should have enough space.
/// assert!(buf.len() >= lz4f::max_compressed_size(data.len(), &prefs));
///
/// let len = lz4f::compress(data, &mut buf, &prefs).unwrap().dst_len();
/// let compressed = &buf[..len];
/// ```
pub fn compress(src: &[u8], dst: &mut [u8], prefs: &Preferences) -> Result<Report> {
    api::compress(src, dst, prefs)
}

/// Read data from a slice and append compressed data to `Vec<u8>`.
///
/// # Examples
///
/// Compress data into the `Vec<u8>` with the default preferences:
/// ```
/// use lzzzz::lz4f;
///
/// let mut buf = Vec::new();
/// lz4f::compress_to_vec(b"Hello world!", &mut buf, &lz4f::Preferences::default());
///
/// let mut buf2 = vec![b'x'];
/// lz4f::decompress_to_vec(&buf, &mut buf2);
/// assert_eq!(buf2.as_slice(), &b"xHello world!"[..]);
/// ```
///
/// This function doesn't clear the content of `Vec<u8>`:
/// ```
/// use lzzzz::lz4f;
///
/// let header = &b"Compressed data:"[..];
/// let mut buf = Vec::from(header);
/// lz4f::compress_to_vec(b"Hello world!", &mut buf, &lz4f::Preferences::default());
/// assert!(buf.starts_with(header));
/// ```
pub fn compress_to_vec(src: &[u8], dst: &mut Vec<u8>, prefs: &Preferences) -> Result<Report> {
    let orig_len = dst.len();
    dst.reserve(max_compressed_size(src.len(), prefs));
    #[allow(unsafe_code)]
    unsafe {
        dst.set_len(dst.capacity());
    }
    let result = compress(src, &mut dst[orig_len..], prefs);
    dst.resize_with(
        orig_len + result.as_ref().map(|r| r.dst_len()).unwrap_or(0),
        Default::default,
    );
    result
}

/// Read data from a slice and append decompressed data to `Vec<u8>`.
pub fn decompress_to_vec(src: &[u8], dst: &mut Vec<u8>) -> Result<Report> {
    let header_len = dst.len();
    let mut src_offset = 0;
    let mut dst_offset = header_len;
    DECOMPRESSION_CTX.with(|ctx| {
        let mut ctx = ctx.borrow_mut();
        ctx.reset();
        loop {
            dst.reserve(1024);
            #[allow(unsafe_code)]
            unsafe {
                dst.set_len(dst.capacity());
            }
            let result =
                ctx.decompress_dict(&src[src_offset..], &mut dst[dst_offset..], &[], false)?;
            src_offset += result.src_len().unwrap();
            dst_offset += result.dst_len();
            let expected = result.expected_src_len().unwrap();
            if expected == 0 {
                dst.resize_with(dst_offset, Default::default);
                return Ok(Report {
                    dst_len: dst_offset - header_len,
                    ..Default::default()
                });
            } else if src_offset >= src.len() {
                return Err(Error::CompressedDataIncomplete);
            }
        }
    })
}

thread_local!(static DECOMPRESSION_CTX: RefCell<DecompressionContext> = RefCell::new(DecompressionContext::new().unwrap()));
