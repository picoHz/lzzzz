use super::{Compressor, CompressorBuilder, Dictionary, Preferences};
use std::{
    convert::TryInto,
    io::{BufRead, Read, Result},
};

pub struct BufReadCompressor<B: BufRead> {
    inner: Compressor<B>,
}

impl<B: BufRead> BufReadCompressor<B> {
    pub(crate) fn new(
        bufreader: B,
        pref: Preferences,
        dict: Option<Dictionary>,
    ) -> crate::Result<Self> {
        Ok(Self {
            inner: Compressor::new(bufreader, pref, None)?,
        })
    }
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

impl<B: BufRead> TryInto<BufReadCompressor<B>> for CompressorBuilder<B> {
    type Error = crate::Error;
    fn try_into(self) -> crate::Result<BufReadCompressor<B>> {
        BufReadCompressor::new(self.device, self.pref, self.dict)
    }
}
