use super::{Decompressor, DecompressorBuilder};
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
}

impl<B: BufRead> Read for BufReadDecompressor<B> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        todo!();
    }
}

impl<B: BufRead> BufRead for BufReadDecompressor<B> {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        todo!();
    }

    fn consume(&mut self, amt: usize) {
        todo!();
    }
}

impl<B: BufRead> TryInto<BufReadDecompressor<B>> for DecompressorBuilder<B> {
    type Error = crate::Error;
    fn try_into(self) -> crate::Result<BufReadDecompressor<B>> {
        BufReadDecompressor::new(self.device)
    }
}
