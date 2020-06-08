//! LZ4 Block Compressor/Decompressor
mod api;

use crate::{LZ4Error, Result};
use api::ExtState;

/// Compression mode specifier
///
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
///     lz4::CompressionMode::Fast { factor: 20 },
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompressionMode {
    /// `Default` is same as `Fast { factor: 1 }`.
    Default,
    /// Custom acceleration factor.
    Fast {
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

/// Calculate the maximum size of the compressed data from the original size.
pub fn max_compressed_size(uncompressed_size: usize) -> usize {
    api::compress_bound(uncompressed_size)
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
/// use lzzzz::lz4;
///
/// let data = b"As soon as they had strength, they arose, joined hands again, and went on.";
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
        CompressionMode::Fast { factor } => factor,
    };
    let len = EXT_STATE
        .with(|state| api::compress_fast_ext_state(&mut state.borrow_mut(), src, dst, acc));
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
/// let data = "En vérité, ne ferait-on pas, pour moins que cela, le Tour du Monde ?";
/// let mut buf = Vec::new();
///
/// lz4::compress(data.as_bytes(), &mut buf, lz4::CompressionMode::Default);
/// ```
///
/// This function doesn't clear the content of `dst`:
/// ```
/// use lzzzz::lz4;
///
/// let header = b"Gladius Dei super terram";
/// let mut buf = Vec::from(&header[..]);
///
/// let data = b"Cito et velociter!";
/// lz4::compress(data, &mut buf, lz4::CompressionMode::Default);
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

/// Decompression mode specifier
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
        uncompressed_size: usize,
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

/// Read data from a slice and write decompressed data into another slice.
///
/// # Examples
///
/// ```
/// use lzzzz::lz4;
///
/// const ORIGINAL_SIZE: usize = 47;
/// let data = [
///     113, 83, 111, 117, 116, 104, 45, 115, 6, 0, 97, 119, 101, 115, 116, 44, 32, 12, 0, 3, 7, 0,
///     48, 45, 101, 97, 19, 0, 160, 101, 97, 115, 116, 46, 32, 46, 46, 46, 32,
/// ];
///
/// let mut buf = [0u8; ORIGINAL_SIZE];
/// lz4::decompress(&data[..], &mut buf[..], lz4::DecompressionMode::Default);
///
/// assert_eq!(
///     &buf[..],
///     "South-south-west, south, south-east, east. ... ".as_bytes()
/// );
/// ```
///
/// ### Partial decompression
///
/// ```
/// use lzzzz::lz4;
///
/// const ORIGINAL_SIZE: usize = 239;
///
/// // Tha latter part of the compressed data is ommited because
/// // we don't need the full plain text.
/// let data = [
///     240, 16, 65, 108, 98, 46, 32, 84, 104, 101, 32, 119, 101, 105, 103, 104, 116, 32, 111, 102,
///     32, 116, 104, 105, 115, 32, 115, 97, 100, 32, 116, 105, 109, 24, 0, 245, 20, 32, 109, 117,
///     115,
/// ];
///
/// let mut buf = [0u8; 30];
/// lz4::decompress(
///     &data[..],
///     &mut buf[..],
///     lz4::DecompressionMode::Partial {
///         uncompressed_size: ORIGINAL_SIZE,
///     },
/// );
///
/// assert_eq!(&buf[..], b"Alb. The weight of this sad ti");
/// ```
pub fn decompress(src: &[u8], dst: &mut [u8], mode: DecompressionMode) -> Result<usize> {
    let len = match mode {
        DecompressionMode::Partial { uncompressed_size } => {
            api::decompress_safe_partial(src, dst, uncompressed_size)
        }
        _ => api::decompress_safe(src, dst),
    };
    if len > 0 {
        Ok(len as usize)
    } else {
        Err(LZ4Error::from("Decompression failed"))
    }
}

thread_local!(static EXT_STATE: ExtState = ExtState::new());
