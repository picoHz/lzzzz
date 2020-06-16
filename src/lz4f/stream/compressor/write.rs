use super::{Compressor, CompressorBuilder, Dictionary, Preferences};
use std::{
    convert::TryInto,
    io::{Result, Write},
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

    fn end(&mut self) -> Result<()> {
        self.inner.end(false)?;
        self.device.write_all(self.inner.buf())?;
        self.inner.clear_buf();
        self.device.flush()
    }
}

impl<W: Write> Write for WriteCompressor<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.inner.update(buf, false)?;
        self.device.write_all(self.inner.buf())?;
        self.inner.clear_buf();
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        self.inner.flush(false)?;
        self.device.write_all(self.inner.buf())?;
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
