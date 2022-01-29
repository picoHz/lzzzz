mod api;

use crate::{common::DICTIONARY_SIZE, lz4, lz4_hc::FavorDecSpeed, Result};
use api::CompressionContext;
use std::{borrow::Cow, cmp, io::Cursor, pin::Pin};

/// Streaming LZ4_HC compressor.
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
/// let mut comp = lz4_hc::Compressor::new()?;
/// let len = comp.next(data, &mut buf)?;
/// let compressed = &buf[..len];
///
/// # let mut buf = [0u8; 256];
/// # let len = lz4::decompress(compressed, &mut buf[..data.len()])?;
/// # assert_eq!(&buf[..len], &data[..]);
/// # Ok::<(), std::io::Error>(())
/// ```
pub struct Compressor<'a> {
    ctx: CompressionContext,
    dict: Pin<Cow<'a, [u8]>>,
    safe_buf: Vec<u8>,
}

impl<'a> Compressor<'a> {
    /// Creates a new `Compressor`.
    pub fn new() -> Result<Self> {
        Ok(Self {
            ctx: CompressionContext::new()?,
            dict: Pin::new(Cow::Borrowed(&[])),
            safe_buf: Vec::new(),
        })
    }

    /// Creates a new `Compressor` with a dictionary.
    pub fn with_dict<D>(dict: D) -> Result<Self>
    where
        D: Into<Cow<'a, [u8]>>,
    {
        let mut comp = Self {
            dict: Pin::new(dict.into()),
            ..Self::new()?
        };
        comp.ctx.load_dict(&comp.dict);
        Ok(comp)
    }

    /// Sets the compression level.
    pub fn set_compression_level(&mut self, level: i32) {
        self.ctx.set_compression_level(level);
    }

    /// Sets the decompression speed mode flag.
    pub fn set_favor_dec_speed(&mut self, dec_speed: FavorDecSpeed) {
        self.ctx
            .set_favor_dec_speed(dec_speed == FavorDecSpeed::Enabled);
    }

    /// Performs LZ4_HC streaming compression.
    ///
    /// Returns the number of bytes written into the destination buffer.
    pub fn next(&mut self, src: &[u8], dst: &mut [u8]) -> Result<usize> {
        self.next_to_ptr(src, dst.as_mut_ptr(), dst.len())
    }

    fn next_to_ptr(&mut self, src: &[u8], dst: *mut u8, dst_len: usize) -> Result<usize> {
        let result = self.ctx.next(src, dst, dst_len)?;
        self.save_dict();
        Ok(result)
    }

    /// Compresses data until the destination slice fills up.
    ///
    /// Returns the number of bytes written into the destination buffer.
    pub fn next_partial<T>(&mut self, src: &mut Cursor<T>, dst: &mut [u8]) -> Result<usize>
    where
        T: AsRef<[u8]>,
    {
        let src_ref = src.get_ref().as_ref();
        let pos = cmp::min(src_ref.len(), src.position() as usize);
        let src_ref = &src_ref[pos..];
        let (src_len, dst_len) = self.ctx.next_partial(src_ref, dst)?;
        src.set_position(src.position() + src_len as u64);
        self.save_dict();
        Ok(dst_len)
    }

    /// Appends a compressed frame to Vec<u8>.
    ///
    /// Returns the number of bytes appended to the given `Vec<u8>`.
    pub fn next_to_vec(&mut self, src: &[u8], dst: &mut Vec<u8>) -> Result<usize> {
        let orig_len = dst.len();
        dst.reserve(lz4::max_compressed_size(src.len()));
        #[allow(unsafe_code)]
        unsafe {
            let result = self.next_to_ptr(
                src,
                dst.as_mut_ptr().add(orig_len),
                dst.capacity() - orig_len,
            );
            dst.set_len(orig_len + result.as_ref().unwrap_or(&0));
            result
        }
    }

    fn save_dict(&mut self) {
        self.safe_buf.resize(DICTIONARY_SIZE, 0);
        self.ctx.save_dict(&mut self.safe_buf);
    }
}
