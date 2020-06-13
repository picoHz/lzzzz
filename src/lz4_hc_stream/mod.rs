//! LZ4_HC Streaming Compressor
//!
//! The `lz4_hc_stream` module doesn't provide decompression functionalities.
//! Use the [`lz4_stream`] module instead.
//!
//! [`lz4_stream`]: ../lz4_stream/index.html
//!
//! # Example
//! ```
//! use lzzzz::lz4_hc_stream;
//!
//! let mut stream = lz4_hc_stream::StreamCompressor::new().unwrap();
//!
//! let data = &b"aaaaa"[..];
//! let mut buf = Vec::new();
//!
//! stream.next_to_vec(data, &mut buf);
//!
//! # use lzzzz::lz4;
//! # let compressed = &buf;
//! # let mut buf = [0u8; 2048];
//! # let len = lz4::decompress(
//! #     compressed,
//! #     &mut buf[..data.len()],
//! #     lz4::DecompressionMode::Default,
//! # )
//! # .unwrap()
//! # .dst_len();
//! # assert_eq!(&buf[..len], &data[..]);
//! ```

mod api;

use crate::{
    lz4, lz4_hc::CompressionLevel, lz4_hc_stream::api::CompressionContext, Error, Report, Result,
};
use std::borrow::Cow;

pub struct StreamCompressor<'a> {
    ctx: CompressionContext,
    prev: Cow<'a, [u8]>,
}

impl<'a> StreamCompressor<'a> {
    pub fn new() -> Result<Self> {
        CompressionContext::new().map(|mut ctx| Self {
            ctx,
            prev: Cow::Borrowed(&[]),
        })
    }

    pub fn set_compression_level(&mut self, level: CompressionLevel) {
        self.ctx.set_compression_level(level.as_i32());
    }

    pub fn set_favor_dec_speed(&mut self, flag: bool) {
        self.ctx.set_favor_dec_speed(flag);
    }

    /// LZ4 Streaming Compressor/Decompressor
    ///
    /// # Example
    /// ```
    /// use lzzzz::{lz4, lz4_hc_stream};
    ///
    /// let mut stream = lz4_hc_stream::StreamCompressor::new().unwrap();
    ///
    /// let data = &b"As soon as they had strength, they arose, joined hands again, and went on."[..];
    /// let mut buf = [0u8; 2048];
    ///
    /// // The slice should have enough space.
    /// assert!(buf.len() >= lz4::max_compressed_size(data.len()));
    ///
    /// let len = stream.next(data, &mut buf).unwrap().dst_len();
    /// let compressed = &buf[..len];
    ///
    /// # let mut buf = [0u8; 2048];
    /// # let len = lz4::decompress(
    /// #     compressed,
    /// #     &mut buf[..data.len()],
    /// #     lz4::DecompressionMode::Default,
    /// # )
    /// # .unwrap()
    /// # .dst_len();
    /// # assert_eq!(&buf[..len], &data[..]);
    /// ```
    pub fn next<S: Into<Cow<'a, [u8]>>>(&mut self, src: S, dst: &mut [u8]) -> Result<Report> {
        let src = src.into();
        let dst_len = self.ctx.next(&src, dst);
        self.prev = src;
        if dst_len > 0 {
            Ok(Report {
                dst_len,
                ..Default::default()
            })
        } else {
            Err(Error::Generic)
        }
    }

    pub fn next_to_vec<S: Into<Cow<'a, [u8]>>>(
        &mut self,
        src: S,
        dst: &mut Vec<u8>,
    ) -> Result<Report> {
        let src = src.into();
        let orig_len = dst.len();
        dst.reserve(lz4::max_compressed_size(src.len()));
        #[allow(unsafe_code)]
        unsafe {
            dst.set_len(dst.capacity());
        }
        let result = self.next(src, &mut dst[orig_len..]);
        dst.resize_with(
            orig_len + result.as_ref().map(|r| r.dst_len()).unwrap_or(0),
            Default::default,
        );
        result
    }
}
