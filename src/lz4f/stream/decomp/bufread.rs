use super::Decompressor;
use crate::lz4f::{FrameInfo, Result};
use std::io::{BufRead, Read};

/// BufRead-based streaming decompressor
///
/// # Examples
///
/// ```
/// # use std::env;
/// # use std::path::Path;
/// # use lzzzz::{Error, Result};
/// # use assert_fs::prelude::*;
/// # let tmp_dir = assert_fs::TempDir::new().unwrap().into_persistent();
/// # env::set_current_dir(tmp_dir.path()).unwrap();
/// #
/// # let mut buf = Vec::new();
/// # lzzzz::lz4f::compress_to_vec(b"Hello world!", &mut buf, &Default::default())?;
/// # tmp_dir.child("foo.lz4").write_binary(&buf).unwrap();
/// #
/// use lzzzz::lz4f::decomp::BufReadDecompressor;
/// use std::{
///     fs::File,
///     io::{prelude::*, BufReader},
/// };
///
/// let mut f = File::open("foo.lz4")?;
/// let mut b = BufReader::new(f);
/// let mut r = BufReadDecompressor::new(&mut b)?;
///
/// let mut buf = Vec::new();
/// r.read_to_end(&mut buf)?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub struct BufReadDecompressor<'a, R: BufRead> {
    device: R,
    inner: Decompressor<'a>,
    consumed: usize,
}

impl<'a, R: BufRead> BufReadDecompressor<'a, R> {
    pub fn new(reader: R) -> Result<Self> {
        Ok(Self {
            device: reader,
            inner: Decompressor::new()?,
            consumed: 0,
        })
    }

    pub fn set_dict<D>(&mut self, dict: D)
    where
        D: AsRef<[u8]> + 'a,
    {
        self.inner.set_dict(dict);
    }

    pub fn read_frame_info(&mut self) -> std::io::Result<FrameInfo> {
        loop {
            if let Some(frame) = self.inner.frame_info() {
                return Ok(frame);
            }
            self.inner.decode_header_only(true);
            let _ = self.read(&mut [])?;
            self.inner.decode_header_only(false);
        }
    }
}

impl<R: BufRead> Read for BufReadDecompressor<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            let inner_buf = self.device.fill_buf()?;
            let consumed = self.inner.decompress(inner_buf)?;
            self.device.consume(consumed);
            if consumed == 0 {
                break;
            }
        }

        let len = std::cmp::min(buf.len(), self.inner.buf().len() - self.consumed);
        buf[..len].copy_from_slice(&self.inner.buf()[self.consumed..][..len]);
        self.consumed += len;
        if self.consumed >= self.inner.buf().len() {
            self.inner.clear_buf();
            self.consumed = 0;
        }
        Ok(len)
    }
}

impl<R: BufRead> BufRead for BufReadDecompressor<'_, R> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        let _ = self.read(&mut [])?;
        Ok(&self.inner.buf()[self.consumed..])
    }

    fn consume(&mut self, amt: usize) {
        self.consumed += amt;
        if self.consumed >= self.inner.buf().len() {
            self.inner.clear_buf();
            self.consumed = 0;
        }
    }
}
