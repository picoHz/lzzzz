use super::{Decompressor, DecompressorBuilder};
use crate::{common::LZ4Error, lz4f::FrameInfo};
use std::{
    convert::TryInto,
    io::{BufRead, Read, Result},
};

pub struct BufReadDecompressor<B: BufRead> {
    device: B,
    inner: Decompressor,
    consumed: usize,
}

impl<B: BufRead> BufReadDecompressor<B> {
    pub(crate) fn new(device: B) -> crate::Result<Self> {
        Ok(Self {
            device,
            inner: Decompressor::new()?,
            consumed: 0,
        })
    }

    pub fn read_frame_info(&mut self) -> Result<FrameInfo> {
        loop {
            match self.inner.get_frame_info() {
                r @ Err(crate::Error::LZ4Error(LZ4Error::FrameHeaderIncomplete)) => {
                    let len = self.inner.buf().len();
                    let _ = self.read(&mut [])?;
                    if self.inner.buf().len() <= len {
                        return Ok(r?);
                    }
                }
                r => return Ok(r?),
            }
        }
    }
}

impl<B: BufRead> Read for BufReadDecompressor<B> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        todo!();
    }
}

impl<B: BufRead> BufRead for BufReadDecompressor<B> {
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

impl<B: BufRead> TryInto<BufReadDecompressor<B>> for DecompressorBuilder<B> {
    type Error = crate::Error;
    fn try_into(self) -> crate::Result<BufReadDecompressor<B>> {
        BufReadDecompressor::new(self.device)
    }
}

#[cfg(test)]
mod tests {
    use crate::lz4f::{decompressor::BufReadDecompressor, DecompressorBuilder};
    use std::{
        fs::File,
        io::{BufReader, Read},
    };

    #[test]
    fn read() -> std::io::Result<()> {
        let mut file = BufReader::new(File::open("README.md")?);
        let mut file = DecompressorBuilder::new(&mut file).build::<BufReadDecompressor<_>>()?;
        panic!("{:?}", file.read_frame_info());
        Ok(())
    }
}
