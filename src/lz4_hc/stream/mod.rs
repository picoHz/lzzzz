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
//! stream.next_to_vec(data, &mut buf);
//!
//! # use lzzzz::lz4;
//! # let compressed = &buf;
//! # let mut buf = [0u8; 2048];
//! # let len = lz4::decompress(
//! #     compressed,
//! #     &mut buf[..data.len()],
//! # )
//! # .unwrap();
//! # assert_eq!(&buf[..len], &data[..]);
//! ```

mod api;

use crate::{common::DICTIONARY_SIZE, lz4, lz4_hc::FavorDecSpeed, Buffer, Result};
use api::CompressionContext;
use std::{borrow::Cow, collections::LinkedList, pin::Pin};

/// Streaming compressor
pub struct Compressor<'a> {
    ctx: CompressionContext,
    cache: LinkedList<Buffer<'a>>,
    cache_len: usize,
    dict: Pin<Cow<'a, [u8]>>,
}

impl<'a> Compressor<'a> {
    pub fn new() -> Result<Self> {
        Ok(Self {
            ctx: CompressionContext::new()?,
            cache: LinkedList::new(),
            cache_len: 0,
            dict: Pin::new(Cow::Borrowed(&[])),
        })
    }

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

    pub fn set_compression_level(&mut self, level: i32) {
        self.ctx.set_compression_level(level);
    }

    pub fn set_favor_dec_speed(&mut self, dec_speed: FavorDecSpeed) {
        self.ctx
            .set_favor_dec_speed(dec_speed == FavorDecSpeed::Enabled);
    }

    pub fn next<B>(&mut self, src: B, dst: &mut [u8]) -> Result<usize>
    where
        B: Into<Buffer<'a>>,
    {
        let src = src.into();
        let result = self.ctx.next(&src, dst)?;
        if !src.is_empty() {
            self.cache_len += src.len();
            self.cache.push_back(src);
        }

        while let Some(len) = self
            .cache
            .front()
            .map(|b| b.len())
            .filter(|n| self.cache_len - n >= DICTIONARY_SIZE)
        {
            self.cache.pop_front();
            self.cache_len -= len;
        }

        Ok(result)
    }

    pub fn next_partial<B>(&mut self, src: B, dst: &mut [u8]) -> Result<(usize, usize)>
    where
        B: Into<Buffer<'a>>,
    {
        let src = src.into();
        let result = self.ctx.next_partial(&src, dst)?;
        if !src.is_empty() {
            self.cache_len += src.len();
            self.cache.push_back(src);
        }

        while let Some(len) = self
            .cache
            .front()
            .map(|b| b.len())
            .filter(|n| self.cache_len - n >= DICTIONARY_SIZE)
        {
            self.cache.pop_front();
            self.cache_len -= len;
        }

        Ok(result)
    }

    pub fn next_to_vec<B>(&mut self, src: B, dst: &mut Vec<u8>) -> Result<usize>
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
        let result = self.next(src, &mut dst[orig_len..]);
        dst.resize_with(orig_len + result.as_ref().unwrap_or(&0), Default::default);
        result
    }
}
