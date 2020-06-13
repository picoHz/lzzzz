//! LZ4 Streaming Compressor/Decompressor
//!
//! # Example
//! ```
//! use lzzzz::lz4;
//!
//! let mut stream = lz4::StreamCompressor::new().unwrap();
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

use crate::{
    lz4,
    lz4::{CompressionMode, DecompressionMode},
    Error, Report, Result,
};
use api::{CompressionContext, DecompressionContext};
use std::borrow::Cow;

pub struct StreamCompressor<'a> {
    ctx: CompressionContext,
    dict: Cow<'a, [u8]>,
    prev: Cow<'a, [u8]>,
}

impl<'a> StreamCompressor<'a> {
    pub fn new() -> Result<Self> {
        CompressionContext::new().map(|ctx| Self {
            ctx,
            dict: Cow::Borrowed(&[]),
            prev: Cow::Borrowed(&[]),
        })
    }

    pub fn reset(&mut self) {
        self.ctx.reset();
    }

    pub fn reset_with_dict(&mut self, dict: Cow<'a, [u8]>) {
        if dict.is_empty() {
            self.reset();
        } else {
            self.ctx.load_dict(&dict);
        }
        self.dict = dict;
    }

    /// LZ4 Streaming Compressor/Decompressor
    ///
    /// # Example
    /// ```
    /// use lzzzz::lz4;
    ///
    /// let mut stream = lz4::StreamCompressor::new().unwrap();
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
        let src_is_empty = src.is_empty();
        let dst_len = self.ctx.next(&src, dst, acc);
        self.prev = src;
        if dst_len > 0 {
            Ok(Report {
                dst_len,
                ..Default::default()
            })
        } else if src_is_empty && dst.is_empty() {
            Ok(Report::default())
        } else {
            Err(Error::CompressionFailed)
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

pub struct StreamDecompressor<'a> {
    ctx: DecompressionContext,
    prev: Cow<'a, [u8]>,
}

impl<'a> StreamDecompressor<'a> {
    pub fn new() -> Result<Self> {
        DecompressionContext::new().map(|ctx| Self {
            ctx,
            prev: Cow::Borrowed(&[]),
        })
    }

    pub fn next<S: Into<Cow<'a, [u8]>>>(
        &mut self,
        src: S,
        dst: &mut [u8],
        mode: DecompressionMode,
    ) -> Result<Report> {
        match mode {
            DecompressionMode::Default => Ok(Default::default()),
            DecompressionMode::Dictionary { data } => Ok(Default::default()),
            _ => Err(Error::DecompressionModeInvalid),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lz4::{CompressionMode, StreamCompressor},
        Error,
    };

    #[test]
    fn empty_dst() {
        assert!(StreamCompressor::new()
            .unwrap()
            .next(&b"hello"[..], &mut [], CompressionMode::Default)
            .is_err());
    }
}
