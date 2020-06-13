//! LZ4 Streaming Compressor/Decompressor
//!
//! # Example
//! ```
//! use lzzzz::{lz4, lz4_stream};
//!
//! let mut stream = lz4_stream::StreamCompressor::new().unwrap();
//!
//! let data = &b"aaaaa"[..];
//! let mut buf = Vec::new();
//!
//! stream.next_to_vec(data, &mut buf, lz4::CompressionMode::Default);
//!
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

use crate::{lz4, lz4::CompressionMode, Error, Report, Result};
use api::CompressionContext;
use std::borrow::Cow;

pub struct StreamCompressor<'a> {
    ctx: CompressionContext,
    dict: Cow<'a, [u8]>,
    prev: Cow<'a, [u8]>,
}

impl<'a> StreamCompressor<'a> {
    pub fn new() -> Result<Self> {
        Self::with_dict(Cow::Borrowed(&[]))
    }

    pub fn with_dict(dict: Cow<'a, [u8]>) -> Result<Self> {
        CompressionContext::new().map(|mut ctx| {
            ctx.set_dict(&dict);
            Self {
                ctx,
                dict,
                prev: Cow::Borrowed(&[]),
            }
        })
    }

    /// LZ4 Streaming Compressor/Decompressor
    ///
    /// # Example
    /// ```
    /// use lzzzz::{lz4, lz4_stream};
    ///
    /// let mut stream = lz4_stream::StreamCompressor::new().unwrap();
    ///
    /// let data = &b"As soon as they had strength, they arose, joined hands again, and went on."[..];
    /// let mut buf = [0u8; 2048];
    ///
    /// // The slice should have enough space.
    /// assert!(buf.len() >= lz4::max_compressed_size(data.len()));
    ///
    /// let len = stream
    ///     .next(data, &mut buf, lz4::CompressionMode::Default)
    ///     .unwrap()
    ///     .dst_len();
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
    pub fn next<S: Into<Cow<'a, [u8]>>>(
        &mut self,
        src: S,
        dst: &mut [u8],
        mode: CompressionMode,
    ) -> Result<Report> {
        let acc = match mode {
            CompressionMode::Default => 1,
            CompressionMode::Acceleration { factor } => factor,
        };
        let src = src.into();
        let dst_len = self.ctx.next(&src, dst, acc);
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
        mode: CompressionMode,
    ) -> Result<Report> {
        let src = src.into();
        let orig_len = dst.len();
        dst.reserve(lz4::max_compressed_size(src.len()));
        #[allow(unsafe_code)]
        unsafe {
            dst.set_len(dst.capacity());
        }
        let result = self.next(src, &mut dst[orig_len..], mode);
        dst.resize_with(
            orig_len + result.as_ref().map(|r| r.dst_len()).unwrap_or(0),
            Default::default,
        );
        result
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn compression_context() {
        use crate::{lz4, lz4_stream};

        let mut stream = lz4_stream::StreamCompressor::new().unwrap();

        let data = &b"aaaaa"[..];
        let mut buf = Vec::new();

        stream.next_to_vec(data, &mut buf, lz4::CompressionMode::Default);

        let compressed = &buf;
        let mut buf = [0u8; 2048];
        let len = lz4::decompress(
            compressed,
            &mut buf[..data.len()],
            lz4::DecompressionMode::Default,
        )
        .unwrap()
        .dst_len();
        assert_eq!(&buf[..len], &data[..]);
    }
}
