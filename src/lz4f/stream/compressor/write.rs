use super::{Compressor, CompressorBuilder, Dictionary, Preferences, State, LZ4F_HEADER_SIZE_MAX};
use std::{
    convert::TryInto,
    io::{IoSlice, Result, Write},
    mem::MaybeUninit,
};

pub struct WriteCompressor<W: Write> {
    device: W,
    inner: Compressor,
}

impl<W: Write> WriteCompressor<W> {
    fn new(writer: W, pref: Preferences, dict: Option<Dictionary>) -> crate::Result<Self> {
        Ok(Self {
            device: writer,
            inner: Compressor::new(pref, dict)?,
        })
    }

    fn ensure_stream(&mut self) -> Result<()> {
        if let State::Created = self.inner.state {
            self.inner.begin()?;
            self.device.write_all(self.inner.buf())?;
            self.inner.state = State::Active;
        }
        Ok(())
    }

    fn write_impl(&mut self, buf: &[u8], stable_src: bool) -> Result<usize> {
        self.ensure_stream()?;
        self.inner.update(buf, stable_src)?;
        self.device.write_all(self.inner.buf())?;
        Ok(buf.len())
    }

    fn end(&mut self) -> Result<()> {
        self.ensure_stream()?;
        match self.inner.state {
            State::Active => {
                self.inner.end(false)?;
                self.device.write_all(self.inner.buf())?;
            }
            _ => unreachable!(),
        }
        self.device.flush()
    }
}

impl<W: Write> Write for WriteCompressor<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.write_impl(buf, false)
    }

    // fn write_vectored(&mut self, bufs: &[IoSlice]) -> Result<usize> {
    // let mut len = 0;
    // for (i, buf) in bufs.iter().enumerate() {
    // let is_last = i + 1 < buf.len();
    // len += self.write_impl(buf, !is_last)?;
    // }
    // Ok(len)
    // }

    fn flush(&mut self) -> Result<()> {
        self.ensure_stream()?;
        match self.inner.state {
            State::Active => {
                self.inner.flush(false)?;
                self.device.write_all(self.inner.buf())?;
            }
            _ => unreachable!(),
        }
        self.device.flush()
    }
}

impl<W: Write> Drop for WriteCompressor<W> {
    fn drop(&mut self) {
        let _ = self.end();
    }
}

impl<W: Write> TryInto<WriteCompressor<W>> for CompressorBuilder<W> {
    type Error = crate::Error;
    fn try_into(self) -> crate::Result<WriteCompressor<W>> {
        WriteCompressor::new(self.device, self.pref, self.dict)
    }
}
