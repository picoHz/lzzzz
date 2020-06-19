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
use api::{CompressionContext, DecompressionContext};
use std::borrow::Cow;
use std::cmp;
use std::collections::LinkedList;

/// Streaming compressor
pub struct Compressor<'a> {
    ctx: CompressionContext,
    dict: Cow<'a, [u8]>,
    prev: Cow<'a, [u8]>,
}

impl<'a> Compressor<'a> {
    pub fn new() -> Result<Self> {
        Ok(Self {
            ctx: CompressionContext::new()?,
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
    /// let mut stream = lz4::Compressor::new().unwrap();
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

/// Streaming decompressor
pub struct Decompressor<'a> {
    ctx: DecompressionContext,
    dict: Cow<'a, [u8]>,
    cache: LinkedList<Vec<u8>>,
    cache_len: usize,
    last_len: usize,
}

impl<'a> Decompressor<'a> {
    pub fn new() -> Result<Self> {
        Ok(Self {
            ctx: DecompressionContext::new()?,
            dict: Cow::Borrowed(&[]),
            cache: LinkedList::new(),
            cache_len: 0,
            last_len: 0,
        })
    }

    pub fn reset(&mut self) -> Result<()> {
        self.reset_with_dict(Cow::Borrowed(&[]))
    }

    pub fn reset_with_dict(&mut self, dict: Cow<'a, [u8]>) -> Result<()> {
        self.ctx.reset(&dict)?;
        self.dict = dict;
        self.cache.clear();
        self.cache_len = 0;
        self.last_len = 0;
        Ok(())
    }

    pub fn next(&mut self, src: &[u8], dst_len: usize) -> Result<Report> {
        if self
            .cache
            .back()
            .map(|v| v.capacity() - v.len())
            .unwrap_or(0)
            < dst_len
        {
            self.cache
                .push_back(Vec::with_capacity(cmp::max(dst_len, 8000)));
        }

        let back = self.cache.back_mut().unwrap();
        let len = back.len();
        #[allow(unsafe_code)]
        unsafe {
            back.set_len(len + dst_len);
        }

        let report = self.ctx.decompress(src, &mut back[len..])?;
        self.last_len = dst_len;

        self.cache_len += report.dst_len();
        let front_len = self.cache.front().map(Vec::len).unwrap_or(0);
        if self.cache_len - front_len >= 64_000 {
            self.cache.pop_front();
            self.cache_len -= front_len;
        }
        Ok(report)
    }

    pub fn data(&self) -> &[u8] {
        if let Some(back) = self.cache.back() {
            let offset = back.len() - self.last_len;
            &back[offset..]
        } else {
            &[]
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lz4::{CompressionMode, Compressor, Decompressor};

    #[test]
    fn empty_dst() {
        assert!(Compressor::new()
            .unwrap()
            .next(&b"hello"[..], &mut [], CompressionMode::Default)
            .is_err());
    }

    #[test]
    fn decompressor() {
        let mut comp = Compressor::new().unwrap();
        let mut decomp = Decompressor::new().unwrap();
        for _ in 0..100 {
            let mut v = Vec::new();
            comp.next_to_vec(
                format!(">>>> xxxxx").as_bytes().to_vec(),
                &mut v,
                CompressionMode::Default,
            )
            .unwrap();
            decomp.next(&v, 10).unwrap();
            println!("<< {:?}", decomp.data());
        }
    }
}
