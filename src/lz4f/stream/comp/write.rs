use super::{Compressor, Dictionary, Preferences};
use crate::lz4f::Result;
use std::io::Write;

/// The [`Write`]-based streaming compressor.
///
/// # Example
///
/// ```
/// # use std::env;
/// # use std::path::Path;
/// # use lzzzz::{Error, Result};
/// # use assert_fs::prelude::*;
/// # let tmp_dir = assert_fs::TempDir::new().unwrap().into_persistent();
/// # env::set_current_dir(tmp_dir.path()).unwrap();
/// use lzzzz::lz4f::WriteCompressor;
/// use std::{fs::File, io::prelude::*};
///
/// let mut f = File::create("foo.lz4")?;
/// let mut w = WriteCompressor::new(&mut f, Default::default())?;
///
/// w.write_all(b"Hello world!")?;
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// [`Write`]: https://doc.rust-lang.org/std/io/trait.Write.html

pub struct WriteCompressor<W: Write> {
    device: Option<W>,
    inner: Compressor,
}

impl<W: Write> WriteCompressor<W> {
    /// Creates a new `WriteCompressor<W>`.
    pub fn new(writer: W, prefs: Preferences) -> Result<Self> {
        Ok(Self {
            device: Some(writer),
            inner: Compressor::new(prefs, None)?,
        })
    }

    /// Creates a new `WriteCompressor<W>` with a dictionary.
    pub fn with_dict(writer: W, prefs: Preferences, dict: Dictionary) -> Result<Self> {
        Ok(Self {
            device: Some(writer),
            inner: Compressor::new(prefs, Some(dict))?,
        })
    }

    /// Returns the ownership of the writer, finishing the stream in the process.
    pub fn into_inner(mut self) -> W {
        let _ = self.end();
        self.device.take().unwrap()
    }

    fn end(&mut self) -> std::io::Result<()> {
        if let Some(device) = &mut self.device {
            self.inner.end(false)?;
            device.write_all(self.inner.buf())?;
            self.inner.clear_buf();
            device.flush()?;
        }

        Ok(())
    }
}

impl<W: Write> Write for WriteCompressor<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.update(buf, false)?;
        self.device.as_mut().unwrap().write_all(self.inner.buf())?;
        self.inner.clear_buf();
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush(false)?;
        self.device.as_mut().unwrap().write_all(self.inner.buf())?;
        self.device.as_mut().unwrap().flush()
    }
}

impl<W: Write> Drop for WriteCompressor<W> {
    fn drop(&mut self) {
        let _ = self.end();
    }
}
