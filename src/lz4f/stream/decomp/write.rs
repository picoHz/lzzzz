use crate::lz4f::{decomp::Decompressor, DecompressorBuilder, FrameInfo};
use std::{
    borrow::Cow,
    convert::TryInto,
    io::{Result, Write},
};

/// Write-based streaming decompressor
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
/// use lzzzz::lz4f::{compress_to_vec, decomp::WriteDecompressor};
/// use std::{fs::File, io::prelude::*};
///
/// let mut f = File::create("foo.txt")?;
/// let mut w = WriteDecompressor::new(&mut f)?;
///
/// let mut buf = Vec::new();
/// compress_to_vec(b"Hello world!", &mut buf, &Default::default())?;
///
/// w.write_all(&buf)?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub struct WriteDecompressor<'a, W: Write> {
    device: W,
    inner: Decompressor<'a>,
}

impl<'a, W: Write> WriteDecompressor<'a, W> {
    pub fn new(writer: W) -> crate::Result<Self> {
        Self::from_builder(writer)
    }

    fn from_builder(device: W) -> crate::Result<Self> {
        Ok(Self {
            device,
            inner: Decompressor::new()?,
        })
    }

    pub fn set_dict(&mut self, dict: Cow<'a, [u8]>) {
        self.inner.set_dict(dict);
    }

    pub fn frame_info(&self) -> Option<FrameInfo> {
        self.inner.frame_info()
    }
}

impl<W: Write> Write for WriteDecompressor<'_, W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let report = self.inner.decompress(buf, false)?;
        self.device.write_all(&self.inner.buf())?;
        self.inner.clear_buf();
        Ok(report.src_len.unwrap())
    }

    fn flush(&mut self) -> Result<()> {
        self.device.flush()
    }
}

impl<'a, W: Write> TryInto<WriteDecompressor<'a, W>> for DecompressorBuilder<W> {
    type Error = crate::Error;
    fn try_into(self) -> crate::Result<WriteDecompressor<'a, W>> {
        WriteDecompressor::from_builder(self.device)
    }
}
