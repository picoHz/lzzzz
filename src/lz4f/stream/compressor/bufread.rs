use super::{Compressor, Dictionary, Preferences};
use crate::lz4f::CompressorBuilder;
use std::{
    convert::TryInto,
    io::{BufRead, Read, Result},
};

/// BufRead-based streaming compressor
pub struct BufReadCompressor<B: BufRead> {
    device: B,
    inner: Compressor,
    consumed: usize,
}

impl<B: BufRead> BufReadCompressor<B> {
    pub(crate) fn new(
        device: B,
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

impl<B: BufRead> Read for BufReadCompressor<B> {
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

impl<B: BufRead> BufRead for BufReadCompressor<B> {
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

impl<B: BufRead> TryInto<BufReadCompressor<B>> for CompressorBuilder<B> {
    type Error = crate::Error;
    fn try_into(self) -> crate::Result<BufReadCompressor<B>> {
        BufReadCompressor::new(self.device, self.pref, self.dict)
    }
}
