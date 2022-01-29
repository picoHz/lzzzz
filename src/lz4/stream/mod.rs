mod api;

use crate::{
    common::{DEFAULT_BUF_SIZE, DICTIONARY_SIZE},
    lz4, Error, ErrorKind, Result,
};
use api::{CompressionContext, DecompressionContext};
use std::{borrow::Cow, cmp, collections::LinkedList, pin::Pin};

/// Streaming LZ4 compressor.
///
/// # Example
///
/// ```
/// use lzzzz::lz4;
///
/// let data = b"The quick brown fox jumps over the lazy dog.";
/// let mut buf = [0u8; 256];
///
/// // The slice should have enough capacity.
/// assert!(buf.len() >= lz4::max_compressed_size(data.len()));
///
/// let mut comp = lz4::Compressor::new()?;
/// let len = comp.next(data, &mut buf, lz4::ACC_LEVEL_DEFAULT)?;
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

    /// Performs LZ4 streaming compression.
    ///
    /// Returns the number of bytes written into the destination buffer.
    pub fn next(&mut self, src: &[u8], dst: &mut [u8], acc: i32) -> Result<usize> {
        self.next_to_ptr(src, dst.as_mut_ptr(), dst.len(), acc)
    }

    fn next_to_ptr(&mut self, src: &[u8], dst: *mut u8, dst_len: usize, acc: i32) -> Result<usize> {
        let is_empty = src.is_empty() && dst_len == 0;
        let dst_len = self.ctx.next(src, dst, dst_len, acc);

        self.save_dict();

        if dst_len > 0 {
            Ok(dst_len)
        } else if is_empty {
            Ok(0)
        } else {
            Err(Error::new(ErrorKind::CompressionFailed))
        }
    }

    /// Appends compressed data to Vec<u8>.
    ///
    /// Returns the number of bytes appended to the given `Vec<u8>`.
    pub fn next_to_vec(&mut self, src: &[u8], dst: &mut Vec<u8>, acc: i32) -> Result<usize> {
        let orig_len = dst.len();
        dst.reserve(lz4::max_compressed_size(src.len()));
        #[allow(unsafe_code)]
        unsafe {
            let result = self.next_to_ptr(
                src,
                dst.as_mut_ptr().add(orig_len),
                dst.capacity() - orig_len,
                acc,
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

/// Streaming LZ4 decompressor.
///
/// # Example
///
/// ```
/// use lzzzz::lz4;
///
/// const ORIGINAL_SIZE: usize = 44;
/// const COMPRESSED_DATA: &str =
///     "8B1UaGUgcXVpY2sgYnJvd24gZm94IGp1bXBzIG92ZXIgdGhlIGxhenkgZG9nLg==";
///
/// let data = base64::decode(COMPRESSED_DATA).unwrap();
///
/// let mut decomp = lz4::Decompressor::new()?;
/// let result = decomp.next(&data[..], ORIGINAL_SIZE)?;
///
/// assert_eq!(result, &b"The quick brown fox jumps over the lazy dog."[..]);
/// # Ok::<(), std::io::Error>(())
/// ```
pub struct Decompressor<'a> {
    ctx: DecompressionContext,
    cache: LinkedList<Vec<u8>>,
    cache_len: usize,
    last_len: usize,
    dict: Pin<Cow<'a, [u8]>>,
}

impl<'a> Decompressor<'a> {
    /// Creates a new `Decompressor`.
    pub fn new() -> Result<Self> {
        Ok(Self {
            ctx: DecompressionContext::new()?,
            cache: LinkedList::new(),
            cache_len: 0,
            last_len: 0,
            dict: Pin::new(Cow::Borrowed(&[])),
        })
    }

    /// Creates a new `Decompressor` with a dictionary.
    pub fn with_dict<D>(dict: D) -> Result<Self>
    where
        D: Into<Cow<'a, [u8]>>,
    {
        let mut decomp = Self {
            dict: Pin::new(dict.into()),
            ..Self::new()?
        };
        decomp.ctx.reset(&decomp.dict)?;
        Ok(decomp)
    }

    /// Decompresses an LZ4 block.
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
        let orig_len = back.len();

        #[allow(unsafe_code)]
        unsafe {
            let dst_len = self.ctx.decompress(
                src,
                back.as_mut_ptr().add(orig_len),
                back.capacity() - orig_len,
            )?;
            back.set_len(orig_len + dst_len);
            self.cache_len += dst_len;
        }
        self.last_len = original_size;

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
