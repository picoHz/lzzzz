mod api;

use super::CompressionLevel;
use crate::{lz4, Error, ErrorKind, Report, Result};
use api::ExtState;

/// Read data from a slice and write compressed data into another slice.
///
/// Ensure that the destination slice have enough capacity.
/// If `dst.len()` is smaller than `lz4::max_compressed_size(src.len())`,
/// this function may fail.
///
/// # Examples
///
/// ### Basic compression
///
/// Compress data with the default compression mode:
///
/// ```
/// use lzzzz::{lz4, lz4_hc};
///
/// let data = "— Да, простите, — повторил он то же слово, которым закончил и весь рассказ.";
/// let mut buf = [0u8; 2048];
///
/// // The slice should have enough capacity.
/// assert!(buf.len() >= lz4::max_compressed_size(data.len()));
///
/// let len = lz4_hc::compress(
///     data.as_bytes(),
///     &mut buf,
///     lz4_hc::CompressionLevel::Default,
/// )?;
///
/// let compressed = &buf[..len];
///
/// # let mut buf = [0u8; 2048];
/// # let len = lz4::decompress(
/// #     compressed,
/// #     &mut buf[..data.len()],
/// #     lz4::DecompressionMode::Default,
/// # )?.dst_len();
/// # assert_eq!(&buf[..len], data.as_bytes());
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn compress(src: &[u8], dst: &mut [u8], compression_level: CompressionLevel) -> Result<usize> {
    if src.is_empty() && dst.is_empty() {
        return Ok(0);
    }
    let result = ExtState::with(|state, reset| {
        if reset {
            api::compress_ext_state_fast_reset(
                &mut state.borrow_mut(),
                src,
                dst,
                compression_level.as_i32(),
            )
        } else {
            api::compress_ext_state(
                &mut state.borrow_mut(),
                src,
                dst,
                compression_level.as_i32(),
            )
        }
    });
    if result.dst_len() > 0 {
        Ok(result.dst_len())
    } else {
        Err(Error::new(ErrorKind::CompressionFailed))
    }
}

pub fn compress_partial(
    src: &[u8],
    dst: &mut [u8],
    compression_level: CompressionLevel,
) -> Result<(usize, usize)> {
    if src.is_empty() && dst.is_empty() {
        return Ok((0, 0));
    }
    let result = ExtState::with(|state, _| {
        api::compress_dest_size(
            &mut state.borrow_mut(),
            src,
            dst,
            compression_level.as_i32(),
        )
    });
    Ok((result.src_len().unwrap(), result.dst_len()))
}

/// Read data from a slice and append compressed data to `Vec<u8>`.
///
/// In this function, [`CompressionMode::Partial`] has no special meaning and
/// is same as [`CompressionMode::Default`].
///
/// [`CompressionMode::Partial`]: ./enum.CompressionMode.html#variant.Partial
/// [`CompressionMode::Default`]: ./enum.CompressionMode.html#variant.Default
///
/// # Examples
///
/// ### Basic usage
///
/// Compress data into the `Vec<u8>` with the default compression mode/level.
///
/// ```
/// use lzzzz::lz4_hc;
///
/// let data = "So we beat on, boats against the current, borne back ceaselessly into the past.";
/// let mut buf = Vec::new();
///
/// lz4_hc::compress_to_vec(data.as_bytes(), &mut buf, lz4_hc::CompressionLevel::Default)?;
///
/// # use lzzzz::lz4;
/// # let compressed = &buf;
/// # let mut buf = [0u8; 2048];
/// # let len = lzzzz::lz4::decompress(
/// #     compressed,
/// #     &mut buf[..data.len()],
/// #     lz4::DecompressionMode::Default,
/// # )?.dst_len();
/// # assert_eq!(&buf[..len], data.as_bytes());
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// ### Higher compression level
///
/// ```
/// use lzzzz::lz4_hc;
///
/// let data = "It was not till they had examined the rings that they recognized who it was.";
/// let mut buf = Vec::new();
///
/// lz4_hc::compress_to_vec(data.as_bytes(), &mut buf, lz4_hc::CompressionLevel::Max)?;
///
/// # use lzzzz::lz4;
/// # let compressed = &buf;
/// # let mut buf = [0u8; 2048];
/// # let len = lzzzz::lz4::decompress(
/// #     compressed,
/// #     &mut buf[..data.len()],
/// #     lz4::DecompressionMode::Default,
/// # )?.dst_len();
/// # assert_eq!(&buf[..len], data.as_bytes());
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn compress_to_vec(
    src: &[u8],
    dst: &mut Vec<u8>,
    compression_level: CompressionLevel,
) -> Result<usize> {
    let orig_len = dst.len();
    dst.reserve(lz4::max_compressed_size(src.len()));
    #[allow(unsafe_code)]
    unsafe {
        dst.set_len(dst.capacity());
    }
    let result = compress(src, &mut dst[orig_len..], compression_level);
    dst.resize_with(orig_len + result.as_ref().unwrap_or(&0), Default::default);
    result
}

#[cfg(test)]
mod tests {
    use crate::lz4_hc::CompressionLevel;

    #[test]
    fn compression_level() {
        assert_eq!(CompressionLevel::Default, CompressionLevel::default());
        assert_eq!(
            CompressionLevel::Min.as_i32(),
            CompressionLevel::Custom(3).as_i32()
        );
        assert_eq!(
            CompressionLevel::Default.as_i32(),
            CompressionLevel::Custom(9).as_i32()
        );
        assert_eq!(
            CompressionLevel::OptMin.as_i32(),
            CompressionLevel::Custom(10).as_i32()
        );
        assert_eq!(
            CompressionLevel::Max.as_i32(),
            CompressionLevel::Custom(12).as_i32()
        );

        let mut sorted = vec![
            CompressionLevel::Custom(std::i32::MIN),
            CompressionLevel::Min,
            CompressionLevel::Default,
            CompressionLevel::OptMin,
            CompressionLevel::Max,
            CompressionLevel::Custom(std::i32::MAX),
        ];
        sorted.sort_unstable();
        assert_eq!(
            sorted,
            vec![
                CompressionLevel::Custom(std::i32::MIN),
                CompressionLevel::Min,
                CompressionLevel::Default,
                CompressionLevel::OptMin,
                CompressionLevel::Max,
                CompressionLevel::Custom(std::i32::MAX),
            ]
        );
    }
}
