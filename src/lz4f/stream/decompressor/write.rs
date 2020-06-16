use crate::lz4f::{decompressor::Decompressor, DecompressorBuilder, FrameInfo};
use std::{
    borrow::Cow,
    convert::TryInto,
    io::{Read, Result, Write},
};

pub struct WriteDecompressor<'a, W: Write> {
    device: W,
    inner: Decompressor<'a>,
}

impl<'a, W: Write> WriteDecompressor<'a, W> {
    pub(crate) fn new(device: W) -> crate::Result<Self> {
        Ok(Self {
            device,
            inner: Decompressor::new()?,
        })
    }

    pub fn set_dict(&mut self, dict: Cow<'a, [u8]>) {
        self.inner.set_dict(dict);
    }

    pub fn frame_info(&self) -> Option<FrameInfo> {
        self.inner.get_frame_info().ok()
    }
}

impl<W: Write> Write for WriteDecompressor<'_, W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let report = self.inner.decompress(buf)?;
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
        WriteDecompressor::new(self.device)
    }
}
