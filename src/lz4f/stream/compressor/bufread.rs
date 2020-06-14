use super::Compressor;
use std::io::{BufRead, Read, Result};

pub struct BufReadCompressor<B: BufRead> {
    ctx: Compressor<B>,
}

impl<B: BufRead> Read for BufReadCompressor<B> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        unimplemented!()
    }
}

impl<B: BufRead> BufRead for BufReadCompressor<B> {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        unimplemented!()
    }

    fn consume(&mut self, amt: usize) {
        unimplemented!()
    }
}
