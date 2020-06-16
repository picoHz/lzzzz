use super::{BufReadDecompressor, DecompressorBuilder};
use crate::lz4f::FrameInfo;
use std::{
    convert::TryInto,
    io::{BufReader, Read, Result},
};

pub struct ReadDecompressor<R: Read> {
    inner: BufReadDecompressor<BufReader<R>>,
}

impl<R: Read> ReadDecompressor<R> {
    pub fn read_frame_info(&mut self) -> Result<FrameInfo> {
        self.inner.read_frame_info()
    }
}

impl<R: Read> Read for ReadDecompressor<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.inner.read(buf)
    }
}

impl<R: Read> TryInto<ReadDecompressor<R>> for DecompressorBuilder<R> {
    type Error = crate::Error;
    fn try_into(self) -> crate::Result<ReadDecompressor<R>> {
        Ok(ReadDecompressor {
            inner: BufReadDecompressor::new(BufReader::new(self.device))?,
        })
    }
}
