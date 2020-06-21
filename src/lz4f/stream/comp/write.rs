use super::{Compressor, Dictionary, Preferences};
use crate::lz4f::{CompressorBuilder, Error, Result};
use std::{convert::TryInto, io::Write};

/// Write-based streaming compressor
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
/// use lzzzz::lz4f::comp::WriteCompressor;
/// use std::{fs::File, io::prelude::*};
///
/// let mut f = File::create("foo.lz4")?;
/// let mut w = WriteCompressor::new(&mut f)?;
///
/// w.write_all(b"hello, world!")?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub struct WriteCompressor<W: Write> {
    device: W,
    inner: Compressor,
}

impl<W: Write> WriteCompressor<W> {
    pub fn new(writer: W) -> Result<Self> {
        CompressorBuilder::new(writer).build()
    }

    fn from_builder(writer: W, pref: Preferences, dict: Option<Dictionary>) -> Result<Self> {
        Ok(Self {
            device: writer,
            inner: Compressor::new(pref, dict)?,
        })
    }

    fn end(&mut self) -> std::io::Result<()> {
        self.inner.end(false)?;
        self.device.write_all(self.inner.buf())?;
        self.inner.clear_buf();
        self.device.flush()
    }
}

impl<W: Write> Write for WriteCompressor<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.update(buf, false)?;
        self.device.write_all(self.inner.buf())?;
        self.inner.clear_buf();
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
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
    type Error = Error;
    fn try_into(self) -> Result<WriteCompressor<W>> {
        WriteCompressor::from_builder(self.device, self.pref, self.dict)
    }
}
