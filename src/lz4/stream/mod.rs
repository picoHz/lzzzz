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
}

impl<'a> Decompressor<'a> {
    pub fn new() -> Result<Self> {
        Ok(Self {
            ctx: DecompressionContext::new()?,
            dict: Cow::Borrowed(&[]),
            cache: LinkedList::new(),
            cache_len: 0,
        })
    }

    pub fn reset(&mut self) -> Result<()> {
        self.ctx.reset(&[])
    }

    pub fn reset_with_dict(&mut self, dict: Cow<'a, [u8]>) -> Result<()> {
        self.ctx.reset(&dict)?;
        self.dict = dict;
        Ok(())
    }

    pub fn next(&mut self, src: &[u8], dst: &mut [u8]) -> Result<Report> {
        if self
            .cache
            .back()
            .map(|v| v.capacity() - v.len())
            .unwrap_or(0)
            < dst.len()
        {
            self.cache
                .push_back(Vec::with_capacity(dst.len().next_power_of_two()));
        }

        let back = self.cache.back_mut().unwrap();
        let len = back.len();
        #[allow(unsafe_code)]
        unsafe {
            back.set_len(len + dst.len());
        }

        let report = self.ctx.decompress(src, &mut back[len..])?;
        self.cache_len += report.dst_len();

        let front_len = self.cache.front().map(|v| v.len()).unwrap_or(0);
        if self.cache_len - front_len >= 64_000 {
            self.cache.pop_front();
            self.cache_len -= front_len;
        }
        Ok(report)
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
            let mut x = vec![0; 10];
            decomp.next(&v, &mut x).unwrap();
        }
    }
}
