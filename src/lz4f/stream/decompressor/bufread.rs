use super::{Decompressor, DecompressorBuilder};
use crate::{common::LZ4Error, lz4f::FrameInfo};
use std::{
    borrow::Cow,
    convert::TryInto,
    io::{BufRead, Read, Result},
};

pub struct BufReadDecompressor<'a, B: BufRead> {
    device: B,
    inner: Decompressor,
    consumed: usize,
    dict: Cow<'a, [u8]>,
}

impl<'a, B: BufRead> BufReadDecompressor<'a, B> {
    pub(crate) fn new(device: B) -> crate::Result<Self> {
        Ok(Self {
            device,
            inner: Decompressor::new()?,
            consumed: 0,
            dict: Cow::Borrowed(&[]),
        })
    }

    pub fn set_dict(&mut self, dict: Cow<'a, [u8]>) {
        self.dict = dict;
    }

    pub fn read_frame_info(&mut self) -> Result<FrameInfo> {
        loop {
            let info = self.inner.get_frame_info();
            if let Err(crate::Error::LZ4Error(LZ4Error::FrameHeaderIncomplete)) = info {
                let len = self.inner.buf().len();
                let _ = self.read(&mut [])?;
                if self.inner.buf().len() > len {
                    continue;
                }
            }
            return Ok(info?);
        }
    }
}

impl<B: BufRead> Read for BufReadDecompressor<'_, B> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        loop {
            let inner_buf = self.device.fill_buf()?;
            if inner_buf.is_empty() {
                break;
            }
            let report = self.inner.decompress(inner_buf, &self.dict)?;
            self.device.consume(report.src_len().unwrap());
            if report.dst_len() > 0 {
                break;
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

impl<B: BufRead> BufRead for BufReadDecompressor<'_, B> {
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

impl<'a, B: BufRead> TryInto<BufReadDecompressor<'a, B>> for DecompressorBuilder<B> {
    type Error = crate::Error;
    fn try_into(self) -> crate::Result<BufReadDecompressor<'a, B>> {
        BufReadDecompressor::new(self.device)
    }
}

#[cfg(test)]
mod tests {
    use crate::lz4f::{
        compressor::WriteCompressor, decompressor::BufReadDecompressor, CompressorBuilder,
        DecompressorBuilder,
    };
    use std::{
        fs::File,
        io::{BufReader, Read, Write},
    };

    #[test]
    fn read() -> std::io::Result<()> {
        {
            let mut file = File::create("foo.lz4")?;
            let mut file = CompressorBuilder::new(&mut file).build::<WriteCompressor<_>>()?;
            file.write_all(b"hello")?;
        }
        {
            let mut file = BufReader::new(File::open("foo.lz4")?);
            let mut file = DecompressorBuilder::new(&mut file).build::<BufReadDecompressor<_>>()?;

            let mut comp = Vec::new();
            file.read_to_end(&mut comp)?;

            println!("{:?}", comp);
            println!("{:?}", file.read_frame_info());
        }
        Ok(())
    }
}
