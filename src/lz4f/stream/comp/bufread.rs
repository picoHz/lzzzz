use super::{Compressor, Dictionary, Preferences};
use crate::lz4f::CompressorBuilder;
use std::{
    convert::TryInto,
    io::{BufRead, Read, Result},
};

/// BufRead-based streaming compressor
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
/// # tmp_dir.child("foo.txt").write_str("Hello").unwrap();
/// #
/// use lzzzz::lz4f::comp::BufReadCompressor;
/// use std::{
///     fs::File,
///     io::{prelude::*, BufReader},
/// };
///
/// let mut f = File::open("foo.txt")?;
/// let mut b = BufReader::new(f);
/// let mut r = BufReadCompressor::new(&mut b)?;
///
/// let mut buf = Vec::new();
/// r.read_to_end(&mut buf)?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub struct BufReadCompressor<R: BufRead> {
    device: R,
    inner: Compressor,
    consumed: usize,
}

impl<R: BufRead> BufReadCompressor<R> {
    pub fn new(reader: R) -> crate::Result<Self> {
        CompressorBuilder::new(reader).build()
    }

    pub(super) fn from_builder(
        device: R,
        pref: Preferences,
        dict: Option<Dictionary>,
    ) -> crate::Result<Self> {
        Ok(Self {
            device,
            inner: Compressor::new(pref, dict)?,
            consumed: 0,
        })
    }
}

impl<R: BufRead> Read for BufReadCompressor<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let consumed = {
            let inner_buf = self.device.fill_buf()?;
            if inner_buf.is_empty() {
                self.inner.end(false)?;
                if self.inner.buf().is_empty() {
                    return Ok(0);
                }
                0
            } else {
                self.inner.update(inner_buf, false)?;
                inner_buf.len()
            }
        };
        self.device.consume(consumed);

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

impl<R: BufRead> BufRead for BufReadCompressor<R> {
    fn fill_buf(&mut self) -> Result<&[u8]> {
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

impl<R: BufRead> TryInto<BufReadCompressor<R>> for CompressorBuilder<R> {
    type Error = crate::Error;
    fn try_into(self) -> crate::Result<BufReadCompressor<R>> {
        BufReadCompressor::from_builder(self.device, self.pref, self.dict)
    }
}
