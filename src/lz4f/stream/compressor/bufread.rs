use super::{Compressor, CompressorBuilder, Dictionary, Preferences, State};
use std::{
    convert::TryInto,
    io::{BufRead, Read, Result},
};

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

    fn ensure_stream(&mut self) -> Result<()> {
        if let State::Created = self.inner.state {
            self.inner.begin()?;
            self.inner.state = State::Active;
        }
        Ok(())
    }
}

impl<B: BufRead> Read for BufReadCompressor<B> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.ensure_stream()?;
        if self.inner.buf().is_empty() {
            if let State::Finished = self.inner.state {
                return Ok(0);
            }
            let buf = self.device.fill_buf()?;
            if buf.is_empty() {
                self.inner.end(false)?;
                self.inner.state = State::Finished;
            } else {
                self.inner.update(buf, false)?;
                let len = buf.len();
                self.device.consume(len);
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
