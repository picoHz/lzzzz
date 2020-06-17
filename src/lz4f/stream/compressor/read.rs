use super::BufReadCompressor;
use crate::lz4f::CompressorBuilder;
use std::{
    convert::TryInto,
    io::{BufReader, Read, Result},
};

/// Read-based streaming compressor
pub struct ReadCompressor<R: Read> {
    inner: BufReadCompressor<BufReader<R>>,
}

impl<R: Read> Read for ReadCompressor<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.inner.read(buf)
    }
}

impl<R: Read> TryInto<ReadCompressor<R>> for CompressorBuilder<R> {
    type Error = crate::Error;
    fn try_into(self) -> crate::Result<ReadCompressor<R>> {
        Ok(ReadCompressor {
            inner: BufReadCompressor::new(BufReader::new(self.device), self.pref, self.dict)?,
        })
    }
}
