mod api;

use crate::{Error, ErrorKind, Result};
use api::ExtState;
use std::cmp;

/// Calculates the maximum size of the compressed data.
///
/// If `original_size` is too large to compress, this returns `0`.
#[must_use]
pub const fn max_compressed_size(original_size: usize) -> usize {
    api::compress_bound(original_size)
}

/// Performs LZ4 block compression.
///
/// Ensure that the destination slice have enough capacity.
/// If `dst.len()` is smaller than `lz4::max_compressed_size(src.len())`,
/// this function may fail.
///
/// Returns the number of bytes written into the destination buffer.
///
/// # Example
///
/// ```
/// use lzzzz::lz4;
///
/// let data = b"The quick brown fox jumps over the lazy dog.";
/// let mut buf = [0u8; 256];
///
/// // The slice should have enough capacity.
/// assert!(buf.len() >= lz4::max_compressed_size(data.len()));
///
/// let len = lz4::compress(data, &mut buf, lz4::ACC_LEVEL_DEFAULT)?;
/// let compressed = &buf[..len];
///
/// # let mut buf = [0u8; 256];
/// # let len = lz4::decompress(compressed, &mut buf[..data.len()])?;
/// # assert_eq!(&buf[..len], &data[..]);
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn compress(src: &[u8], dst: &mut [u8], acc: i32) -> Result<usize> {
    if src.is_empty() {
        return Ok(0);
    }

    // Workaround for https://github.com/lz4/lz4/issues/876
    let acc = cmp::min(acc, 33_554_431);

    let len = ExtState::with(|state, reset| {
        let mut state = state.borrow_mut();
        if reset {
            api::compress_fast_ext_state_fast_reset(&mut state, src, dst, acc)
        } else {
            api::compress_fast_ext_state(&mut state, src, dst, acc)
        }
    });
    if len > 0 {
        Ok(len)
    } else {
        Err(Error::new(ErrorKind::DecompressionFailed))
    }
}

/// Appends compressed data to `Vec<u8>`.
///
/// Returns the number of bytes appended to `Vec<u8>`.
///
/// # Example
///
/// ```
/// use lzzzz::lz4;
///
/// let data = b"The quick brown fox jumps over the lazy dog.";
/// let mut buf = Vec::new();
///
/// lz4::compress_to_vec(data, &mut buf, lz4::ACC_LEVEL_DEFAULT)?;
/// # let compressed = &buf;
/// # let mut buf = [0u8; 256];
/// # let len = lz4::decompress(compressed, &mut buf[..data.len()])?;
/// # assert_eq!(&buf[..len], &data[..]);
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn compress_to_vec(src: &[u8], dst: &mut Vec<u8>, acc: i32) -> Result<usize> {
    let orig_len = dst.len();
    dst.reserve(max_compressed_size(src.len()));
    #[allow(unsafe_code)]
    unsafe {
        dst.set_len(dst.capacity());
    };
    let result = compress(src, &mut dst[orig_len..], acc);
    dst.resize_with(orig_len + result.as_ref().unwrap_or(&0), Default::default);
    result
}

/// Decompresses a LZ4 block.
///
/// The length of the destination slice must be equal to the original data length.
///
/// Returns the number of bytes written into the destination buffer.
///
/// # Example
///
/// ```
/// use lzzzz::lz4;
///
/// const ORIGINAL_SIZE: usize = 44;
/// const COMPRESSED_DATA: &str =
///     "8B1UaGUgcXVpY2sgYnJvd24gZm94IGp1bXBzIG92ZXIgdGhlIGxhenkgZG9nLg==";
///
/// let data = base64::decode(COMPRESSED_DATA).unwrap();
/// let mut buf = [0u8; ORIGINAL_SIZE];
///
/// lz4::decompress(&data[..], &mut buf[..])?;
///
/// assert_eq!(
///     &buf[..],
///     &b"The quick brown fox jumps over the lazy dog."[..]
/// );
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn decompress(src: &[u8], dst: &mut [u8]) -> Result<usize> {
    api::decompress_safe(src, dst)
}

/// Decompresses a LZ4 block until the destination slice fills up.
///
/// Returns the number of bytes written into the destination buffer.
///
/// # Example
///
/// ```
/// use lzzzz::lz4;
///
/// const ORIGINAL_SIZE: usize = 44;
/// const COMPRESSED_DATA: &str =
///     "8B1UaGUgcXVpY2sgYnJvd24gZm94IGp1bXBzIG92ZXIgdGhlIGxhenkgZG9nLg==";
///
/// let data = base64::decode(COMPRESSED_DATA).unwrap();
/// let mut buf = [0u8; 24];
///
/// lz4::decompress_partial(&data[..], &mut buf[..], ORIGINAL_SIZE)?;
///
/// assert_eq!(&buf[..], &b"The quick brown fox jump"[..]);
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn decompress_partial(src: &[u8], dst: &mut [u8], original_size: usize) -> Result<usize> {
    api::decompress_safe_partial(src, dst, original_size)
}

/// Decompresses a LZ4 block with a dictionary.
///
/// Returns the number of bytes written into the destination buffer.
///
/// # Example
///
/// ```
/// use lzzzz::lz4;
///
/// const ORIGINAL_SIZE: usize = 44;
/// const COMPRESSED_DATA: &str = "DywAFFAgZG9nLg==";
/// const DICT_DATA: &[u8] = b"The quick brown fox jumps over the lazy cat.";
///
/// let data = base64::decode(COMPRESSED_DATA).unwrap();
/// let mut buf = [0u8; ORIGINAL_SIZE];
///
/// lz4::decompress_with_dict(&data[..], &mut buf[..], DICT_DATA)?;
///
/// assert_eq!(
///     &buf[..],
///     &b"The quick brown fox jumps over the lazy dog."[..]
/// );
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn decompress_with_dict(src: &[u8], dst: &mut [u8], dict: &[u8]) -> Result<usize> {
    api::decompress_safe_using_dict(src, dst, dict)
}
