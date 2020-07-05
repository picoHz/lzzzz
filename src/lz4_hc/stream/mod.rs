//! LZ4_HC Streaming Compressor
//!
//! The `lz4_hc_stream` module doesn't provide decompression functionalities.
//! Use the [`lz4_stream`] module instead.
//!
//! [`lz4_stream`]: ../lz4_stream/index.html
//!
//! # Example
//! ```
//! use lzzzz::lz4_hc;
//!
//! let mut stream = lz4_hc::Compressor::new().unwrap();
//!
//! let data = &b"aaaaa"[..];
//! let mut buf = Vec::new();
//!
//! stream.next_to_vec(data, &mut buf, lz4_hc::CompressionMode::Default);
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
    lz4,
    lz4_hc::{CompressionLevel, CompressionMode},
    Buffer, Report, Result,
};
use api::CompressionContext;
use std::collections::LinkedList;

/// Streaming compressor
pub struct Compressor<'a> {
    ctx: CompressionContext,
    compression_level: CompressionLevel,
    dict: Buffer<'a>,
    cache: LinkedList<Buffer<'a>>,
    cache_len: usize,
}

impl<'a> Compressor<'a> {
    pub fn new() -> Result<Self> {
        Ok(Self {
            ctx: CompressionContext::new()?,
            compression_level: CompressionLevel::Default,
            dict: Buffer::new(),
            cache: LinkedList::new(),
            cache_len: 0,
        })
    }

    pub fn with_dict<B>(dict: B) -> Result<Self>
    where
        B: Into<Buffer<'a>>,
    {
        let mut comp = Self::new()?;
        comp.reset_with_dict(dict);
        Ok(comp)
    }

    pub fn reset(&mut self) {
        self.ctx.reset(self.compression_level.as_i32());
    }

    pub fn reset_with_dict<B>(&mut self, dict: B)
    where
        B: Into<Buffer<'a>>,
    {
        let dict = dict.into();
        self.ctx.load_dict(&dict);
        self.dict = dict;
    }

    pub fn set_compression_level(&mut self, level: CompressionLevel) {
        self.compression_level = level;
        self.ctx.set_compression_level(level.as_i32());
    }

    pub fn set_favor_dec_speed(&mut self, flag: bool) {
        self.ctx.set_favor_dec_speed(flag);
    }

    pub fn next<B>(&mut self, src: B, dst: &mut [u8], mode: CompressionMode) -> Result<Report>
    where
        B: Into<Buffer<'a>>,
    {
        let src = src.into();
        let result = match mode {
            CompressionMode::Default => self.ctx.next(&src, dst),
            _ => self.ctx.next_partial(&src, dst),
        };
        if !src.is_empty() {
            self.cache_len += src.len();
            self.cache.push_back(src);
        }

        while let Some(len) = self
            .cache
            .front()
            .map(|b| b.len())
            .filter(|n| self.cache_len - n >= 64_000)
        {
            self.cache.pop_front();
            self.cache_len -= len;
        }

        result
    }

    pub fn next_to_vec<B>(
        &mut self,
        src: B,
        dst: &mut Vec<u8>,
        mode: CompressionMode,
    ) -> Result<Report>
    where
        B: Into<Buffer<'a>>,
    {
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
