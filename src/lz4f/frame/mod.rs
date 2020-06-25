//! LZ4 Frame Compressor/Decompressor

use super::{api, Result};
use crate::{common::DEFAULT_BUF_SIZE, lz4f::Preferences, Error, ErrorKind, Report};
use std::{cell::RefCell, mem::MaybeUninit, ops::Deref};

/// Calculate the maximum size of the compressed data from the original size.
pub fn max_compressed_size(original_size: usize, prefs: &Preferences) -> usize {
    api::compress_frame_bound(original_size, prefs)
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
    #[allow(unsafe_code)]
    unsafe {
        dst.resize(
            orig_len + max_compressed_size(src.len(), prefs),
            MaybeUninit::uninit().assume_init(),
        );
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
    DecompressionCtx::with(|ctx| {
        let mut ctx = ctx.borrow_mut();
        ctx.reset();
        loop {
            #[allow(unsafe_code)]
            unsafe {
                dst.resize(
                    dst.len() + DEFAULT_BUF_SIZE,
                    MaybeUninit::uninit().assume_init(),
                );
            }
            match ctx.decompress_dict(&src[src_offset..], &mut dst[dst_offset..], &[], false) {
                Ok((result, expected)) => {
                    src_offset += result.src_len().unwrap();
                    dst_offset += result.dst_len();
                    if expected == 0 {
                        dst.resize_with(dst_offset, Default::default);
                        return Ok(Report {
                            dst_len: dst_offset - header_len,
                            ..Default::default()
                        });
                    } else if src_offset >= src.len() {
                        dst.resize_with(header_len, Default::default);
                        return Err(Error::new(ErrorKind::CompressedDataIncomplete).into());
                    }
                }
                Err(err) => {
                    dst.resize_with(header_len, Default::default);
                    return Err(err);
                }
            }
        }
    })
}

struct DecompressionCtx(RefCell<api::DecompressionContext>);

impl DecompressionCtx {
    fn new() -> Self {
        Self(RefCell::new(api::DecompressionContext::new().unwrap()))
    }

    fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&RefCell<api::DecompressionContext>) -> R,
    {
        #[cfg(feature = "use-tls")]
        {
            DECOMPRESSION_CTX.with(|state| (f)(state))
        }

        #[cfg(not(feature = "use-tls"))]
        {
            (f)(&Self::new())
        }
    }
}

impl Deref for DecompressionCtx {
    type Target = RefCell<api::DecompressionContext>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "use-tls")]
thread_local!(static DECOMPRESSION_CTX: DecompressionCtx = DecompressionCtx::new());
