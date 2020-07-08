//! LZ4 Block Compressor/Decompressor
mod api;

use crate::{Error, ErrorKind, Report, Result};
use api::ExtState;

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

/// Calculate the maximum size of the compressed data from the original size.
///
/// If `original_size` is too large to compress, this function returns `0`.
#[must_use]
pub const fn max_compressed_size(original_size: usize) -> usize {
    api::compress_bound(original_size)
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
/// // The slice should have enough capacity.
/// assert!(buf.len() >= lz4::max_compressed_size(data.len()));
///
/// let len = lz4::compress(data, &mut buf, lz4::CompressionMode::Default)
///     .unwrap()
///     .dst_len();
/// let compressed = &buf[..len];
///
/// # let mut buf = [0u8; 2048];
/// # let len = lz4::decompress(compressed,
/// #    &mut buf[..data.len()],
/// #    lz4::DecompressionMode::Default).unwrap().dst_len();
/// # assert_eq!(&buf[..len], &data[..]);
/// ```
pub fn compress(src: &[u8], dst: &mut [u8], mode: CompressionMode) -> Result<Report> {
    let acc = match mode {
        CompressionMode::Default => 1,
        CompressionMode::Acceleration { factor } => factor,
    };
    let len = ExtState::with(|state, reset| {
        let mut state = state.borrow_mut();
        if reset {
            api::compress_fast_ext_state_fast_reset(&mut state, src, dst, acc)
        } else {
            api::compress_fast_ext_state(&mut state, src, dst, acc)
        }
    });
    if len.dst_len() > 0 {
        Ok(len)
    } else if src.is_empty() && dst.is_empty() {
        Ok(Report::default())
    } else {
        Err(Error::new(ErrorKind::DecompressionFailed))
    }
}

/// Read data from a slice and append compressed data to `Vec<u8>`.
///
/// # Examples
///
/// ### Basic usage
///
/// Compress data into the `Vec<u8>` with the default compression mode.
///
/// ```
/// use lzzzz::lz4;
///
/// let data = "En vérité, ne ferait-on pas, pour moins que cela, le Tour du Monde ?";
/// let mut buf = Vec::new();
///
/// lz4::compress_to_vec(data.as_bytes(), &mut buf, lz4::CompressionMode::Default);
/// # let compressed = &buf;
/// # let mut buf = [0u8; 2048];
/// # let len = lz4::decompress(compressed,
/// #    &mut buf[..data.len()],
/// #    lz4::DecompressionMode::Default).unwrap().dst_len();
/// # assert_eq!(&buf[..len], data.as_bytes());
/// ```
///
/// ### Preserving header
///
/// This function doesn't clear the content of `dst`.
///
/// ```
/// use lzzzz::lz4;
///
/// let header = b"Gladius Dei super terram";
/// let mut buf = Vec::from(&header[..]);
///
/// let data = b"Cito et velociter!";
/// lz4::compress_to_vec(data, &mut buf, lz4::CompressionMode::Default);
/// assert!(buf.starts_with(header) && buf.len() > header.len());
///
/// # let compressed = &buf[header.len()..];
/// # let mut buf = [0u8; 2048];
/// # let len = lz4::decompress(compressed,
/// #    &mut buf[..data.len()],
/// #    lz4::DecompressionMode::Default).unwrap().dst_len();
/// # assert_eq!(&buf[..len], &data[..]);
/// ```
///
/// ### Accelerated compression mode
///
/// Faster, but less effective compression.
/// See [`CompressionMode::Acceleration`] for details.
///
/// [`CompressionMode::Acceleration`]:
/// ./enum.CompressionMode.html#variant.Acceleration
///
/// ```
/// use lzzzz::lz4;
///
/// let data = b"QUATRE HEURES.";
/// let mut buf = Vec::new();
///
/// lz4::compress_to_vec(
///     data,
///     &mut buf,
///     lz4::CompressionMode::Acceleration { factor: 20 },
/// );
///
/// # let compressed = &buf;
/// # let mut buf = [0u8; 2048];
/// # let len = lz4::decompress(compressed,
/// #    &mut buf[..data.len()],
/// #    lz4::DecompressionMode::Default).unwrap().dst_len();
/// # assert_eq!(&buf[..len], &data[..]);
/// ```
pub fn compress_to_vec(src: &[u8], dst: &mut Vec<u8>, mode: CompressionMode) -> Result<Report> {
    let orig_len = dst.len();
    dst.reserve(max_compressed_size(src.len()));
    #[allow(unsafe_code)]
    unsafe {
        dst.set_len(dst.capacity());
    };
    let result = compress(src, &mut dst[orig_len..], mode);
    dst.resize_with(
        orig_len + result.as_ref().map(|r| r.dst_len()).unwrap_or(0),
        Default::default,
    );
    result
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

/// Read data from a slice and write decompressed data into another slice.
///
/// # Examples
///
/// ### Basic usage
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
///     &b"South-south-west, south, south-east, east. ... "[..]
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
///         original_size: ORIGINAL_SIZE,
///     },
/// );
///
/// assert_eq!(&buf[..], b"Alb. The weight of this sad ti");
/// ```
pub fn decompress(src: &[u8], dst: &mut [u8], mode: DecompressionMode) -> Result<Report> {
    match mode {
        DecompressionMode::Default => api::decompress_safe(src, dst),
        DecompressionMode::Partial { original_size } => {
            api::decompress_safe_partial(src, dst, original_size)
        }
        DecompressionMode::Dictionary { data } => api::decompress_safe_using_dict(src, dst, data),
    }
}
