use super::{BufReadCompressor, CompressorBuilder};
use std::{
    convert::TryInto,
    io::{BufReader, Read, Result},
};

pub struct ReadCompressor<R: Read> {
    bufread: BufReadCompressor<BufReader<R>>,
}

impl<R: Read> Read for ReadCompressor<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.bufread.read(buf)
    }
}

impl<R: Read> TryInto<ReadCompressor<R>> for CompressorBuilder<R> {
    type Error = crate::Error;
    fn try_into(self) -> crate::Result<ReadCompressor<R>> {
        Ok(ReadCompressor {
            bufread: BufReadCompressor::new(BufReader::new(self.device), self.pref, self.dict)?,
        })
    }
}
