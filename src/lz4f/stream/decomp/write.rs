use crate::{
    lz4f::{decomp::Decompressor, DecompressorBuilder, Error, FrameInfo, Result},
    Buffer,
};
use std::{convert::TryInto, io::Write};

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
    pub fn new(writer: W) -> Result<Self> {
        DecompressorBuilder::new(writer).build()
    }

    fn from_builder(device: W, capacity: usize) -> Result<Self> {
        Ok(Self {
            device,
            inner: Decompressor::new(capacity)?,
        })
    }

    pub fn set_dict<B>(&mut self, dict: B)
    where
        B: Into<Buffer<'a>>,
    {
        self.inner.set_dict(dict.into());
    }

    pub fn frame_info(&self) -> Option<FrameInfo> {
        self.inner.frame_info()
    }

    pub fn decode_header_only(&mut self, flag: bool) {
        self.inner.decode_header_only(flag);
    }
}

impl<W: Write> Write for WriteDecompressor<'_, W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let report = self.inner.decompress(buf)?;
        self.device.write_all(self.inner.buf())?;
        self.inner.clear_buf();
        Ok(report.src_len.unwrap())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.device.flush()
    }
}

impl<'a, W: Write> TryInto<WriteDecompressor<'a, W>> for DecompressorBuilder<W> {
    type Error = Error;
    fn try_into(self) -> Result<WriteDecompressor<'a, W>> {
        WriteDecompressor::from_builder(self.device, self.capacity)
    }
}
