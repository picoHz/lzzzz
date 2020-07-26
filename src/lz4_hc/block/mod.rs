mod api;

use crate::{lz4, Error, ErrorKind, Result};
use api::ExtState;

/// Performs LZ4_HC block compression.
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
/// use lzzzz::{lz4, lz4_hc};
///
/// let data = b"The quick brown fox jumps over the lazy dog.";
/// let mut buf = [0u8; 256];
///
/// // The slice should have enough capacity.
/// assert!(buf.len() >= lz4::max_compressed_size(data.len()));
///
/// let len = lz4_hc::compress(data, &mut buf, lz4_hc::CLEVEL_DEFAULT)?;
/// let compressed = &buf[..len];
///
/// # let mut buf = [0u8; 256];
/// # let len = lz4::decompress(
/// #     compressed,
/// #     &mut buf[..data.len()],
/// # )?;
/// # assert_eq!(&buf[..len], &data[..]);
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn compress(src: &[u8], dst: &mut [u8], level: i32) -> Result<usize> {
    if src.is_empty() {
        return Ok(0);
    }
    let len = ExtState::with(|state, reset| {
        if reset {
            api::compress_ext_state_fast_reset(&mut state.borrow_mut(), src, dst, level)
        } else {
            api::compress_ext_state(&mut state.borrow_mut(), src, dst, level)
        }
    });
    if len > 0 {
        Ok(len)
    } else {
        Err(Error::new(ErrorKind::CompressionFailed))
    }
}

/// Compresses data until the destination slice fills up.
///
/// The first `usize` of the returned value represents the number of bytes written into the
/// destination buffer, and the other represents the number of bytes read from the source buffer.
///
/// # Example
///
/// ```
/// use lzzzz::{lz4, lz4_hc};
///
/// let data = b"The quick brown fox jumps over the lazy dog.";
/// let mut buf = [0u8; 16];
///
/// let (src_len, dst_len) = lz4_hc::compress_partial(data, &mut buf, lz4_hc::CLEVEL_DEFAULT)?;
/// let compressed = &buf[..dst_len];
///
/// # let mut buf = [0u8; 256];
/// # let len = lz4::decompress(
/// #     compressed,
/// #     &mut buf[..data.len()],
/// # )?;
/// # assert_eq!(&buf[..len], &data[..src_len]);
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn compress_partial(src: &[u8], dst: &mut [u8], level: i32) -> Result<(usize, usize)> {
    if src.is_empty() || dst.is_empty() {
        return Ok((0, 0));
    }
    Ok(ExtState::with(|state, _| {
        api::compress_dest_size(&mut state.borrow_mut(), src, dst, level)
    }))
}

/// Appends compressed data to `Vec<u8>`.
///
/// Returns the number of bytes appended to `Vec<u8>`.
///
/// # Example
///
/// ```
/// use lzzzz::{lz4, lz4_hc};
///
/// let data = b"The quick brown fox jumps over the lazy dog.";
/// let mut buf = Vec::new();
///
/// lz4_hc::compress_to_vec(data, &mut buf, lz4_hc::CLEVEL_DEFAULT)?;
///
/// # let compressed = &buf;
/// # let mut buf = [0u8; 256];
/// # let len = lz4::decompress(
/// #     compressed,
/// #     &mut buf[..data.len()],
/// # )?;
/// # assert_eq!(&buf[..len], &data[..]);
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn compress_to_vec(src: &[u8], dst: &mut Vec<u8>, level: i32) -> Result<usize> {
    let orig_len = dst.len();
    dst.reserve(lz4::max_compressed_size(src.len()));
    #[allow(unsafe_code)]
    unsafe {
        dst.set_len(dst.capacity());
    }
    let result = compress(src, &mut dst[orig_len..], level);
    dst.resize_with(orig_len + result.as_ref().unwrap_or(&0), Default::default);
    result
}
