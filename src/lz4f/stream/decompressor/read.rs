use super::BufReadDecompressor;
use crate::lz4f::{DecompressorBuilder, FrameInfo};
use std::{
    borrow::Cow,
    convert::TryInto,
    io::{BufReader, Read, Result},
};

pub struct ReadDecompressor<'a, R: Read> {
    inner: BufReadDecompressor<'a, BufReader<R>>,
}

impl<'a, R: Read> ReadDecompressor<'a, R> {
    pub fn read_frame_info(&mut self) -> Result<FrameInfo> {
        self.inner.read_frame_info()
    }

    pub fn set_dict(&mut self, dict: Cow<'a, [u8]>) {
        self.inner.set_dict(dict);
    }
}

impl<R: Read> Read for ReadDecompressor<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.inner.read(buf)
    }
}

impl<'a, R: Read> TryInto<ReadDecompressor<'a, R>> for DecompressorBuilder<R> {
    type Error = crate::Error;
    fn try_into(self) -> crate::Result<ReadDecompressor<'a, R>> {
        Ok(ReadDecompressor {
            inner: BufReadDecompressor::new(BufReader::new(self.device))?,
        })
    }
}
