//! LZ4 Streaming Compressor/Decompressor
//!
//! # Example
//! ```
//! use lzzzz::lz4;
//!
//! let mut stream = lz4::Compressor::new().unwrap();
//!
//! let data = &b"aaaaa"[..];
//! let mut buf = Vec::new();
//!
//! stream.next_to_vec(data, &mut buf, lz4::ACCELERATION_DEFAULT);
//!
//! # let compressed = &buf;
//! # let mut buf = [0u8; 2048];
//! # let len = lz4::decompress(
//! #     compressed,
//! #     &mut buf[..data.len()]
//! # )
//! # .unwrap();
//! # assert_eq!(&buf[..len], &data[..]);
//! ```

mod api;

use crate::{
    common::{DEFAULT_BUF_SIZE, DICTIONARY_SIZE},
    lz4, Buffer, Error, ErrorKind, Result,
};
use api::{CompressionContext, DecompressionContext};
use std::{cmp, collections::LinkedList};

/// Streaming compressor
pub struct Compressor<'a> {
    ctx: CompressionContext,
    dict: Buffer<'a>,
    cache: LinkedList<Buffer<'a>>,
    cache_len: usize,
}

impl<'a> Compressor<'a> {
    pub fn new() -> Result<Self> {
        Ok(Self {
            ctx: CompressionContext::new()?,
            dict: Buffer::new(),
            cache: LinkedList::new(),
            cache_len: 0,
        })
    }

    pub fn with_dict<B>(dict: B) -> Result<Self>
    where
        B: Into<Buffer<'a>>,
    {
        let dict = dict.into();
        let mut comp = Self::new()?;
        comp.ctx.load_dict(&dict);
        comp.dict = dict;
        Ok(comp)
    }

    /// LZ4 Streaming Compressor/Decompressor
    ///
    /// # Example
    /// ```
    /// use lzzzz::lz4;
    ///
    /// let mut stream = lz4::Compressor::new().unwrap();
    ///
    /// let data = &b"As soon as they had strength, they arose, joined hands again, and went on."[..];
    /// let mut buf = [0u8; 2048];
    ///
    /// // The slice should have enough capacity.
    /// assert!(buf.len() >= lz4::max_compressed_size(data.len()));
    ///
    /// let len = stream
    ///     .next(data, &mut buf, lz4::ACCELERATION_DEFAULT)
    ///     .unwrap();
    /// let compressed = &buf[..len];
    ///
    /// # let mut buf = [0u8; 2048];
    /// # let len = lz4::decompress(
    /// #     compressed,
    /// #     &mut buf[..data.len()]
    /// # )
    /// # .unwrap();
    /// # assert_eq!(&buf[..len], &data[..]);
    /// ```
    pub fn next<B>(&mut self, src: B, dst: &mut [u8], acc: i32) -> Result<usize>
    where
        B: Into<Buffer<'a>>,
    {
        let src = src.into();
        let src_is_empty = src.is_empty();
        let dst_len = self.ctx.next(&src, dst, acc);

        if !src_is_empty {
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

        if dst_len > 0 {
            Ok(dst_len)
        } else if src_is_empty && dst.is_empty() {
            Ok(0)
        } else {
            Err(Error::new(ErrorKind::CompressionFailed))
        }
    }

    pub fn next_to_vec<B>(&mut self, src: B, dst: &mut Vec<u8>, acc: i32) -> Result<usize>
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
        let result = self.next(src, &mut dst[orig_len..], acc);
        dst.resize_with(orig_len + result.as_ref().unwrap_or(&0), Default::default);
        result
    }
}

/// Streaming decompressor
pub struct Decompressor<'a> {
    ctx: DecompressionContext,
    dict: Buffer<'a>,
    cache: LinkedList<Vec<u8>>,
    cache_len: usize,
    last_len: usize,
}

impl<'a> Decompressor<'a> {
    pub fn new() -> Result<Self> {
        Ok(Self {
            ctx: DecompressionContext::new()?,
            dict: Buffer::new(),
            cache: LinkedList::new(),
            cache_len: 0,
            last_len: 0,
        })
    }

    pub fn with_dict<B>(dict: B) -> Result<Self>
    where
        B: Into<Buffer<'a>>,
    {
        let dict = dict.into();
        let mut decomp = Self::new()?;
        decomp.ctx.reset(&dict)?;
        decomp.dict = dict;
        Ok(decomp)
    }

    pub fn next(&mut self, src: &[u8], original_size: usize) -> Result<&[u8]> {
        if self
            .cache
            .back()
            .map(|v| v.capacity() - v.len())
            .filter(|n| *n >= original_size)
            .is_none()
        {
            self.cache.push_back(Vec::with_capacity(cmp::max(
                original_size,
                DEFAULT_BUF_SIZE,
            )));
        }

        let back = self.cache.back_mut().unwrap();
        let len = back.len();
        #[allow(unsafe_code)]
        unsafe {
            back.set_len(len + original_size);
        }

        let dst_len = self.ctx.decompress(src, &mut back[len..])?;
        self.last_len = original_size;

        self.cache_len += dst_len;
        while let Some(len) = self
            .cache
            .front()
            .map(Vec::len)
            .filter(|n| self.cache_len - n >= DICTIONARY_SIZE)
        {
            self.cache.pop_front();
            self.cache_len -= len;
        }
        Ok(self.data())
    }

    fn data(&self) -> &[u8] {
        if let Some(back) = self.cache.back() {
            let offset = back.len() - self.last_len;
            &back[offset..]
        } else {
            &[]
        }
    }
}
