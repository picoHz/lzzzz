use super::{Compressor, CompressorBuilder, Dictionary, Preferences};
use std::{
    convert::TryInto,
    io::{BufRead, Read, Result},
};

pub struct BufReadCompressor<B: BufRead> {
    device: B,
    inner: Compressor,
    consumed: usize,
}

impl<B: BufRead> BufReadCompressor<B> {
    pub(crate) fn new(
        device: B,
        pref: Preferences,
        dict: Option<Dictionary>,
    ) -> crate::Result<Self> {
        Ok(Self {
            device,
            inner: Compressor::new(pref, None)?,
            consumed: 0,
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
        self.consumed += amt;
        unimplemented!()
    }
}

impl<B: BufRead> TryInto<BufReadCompressor<B>> for CompressorBuilder<B> {
    type Error = crate::Error;
    fn try_into(self) -> crate::Result<BufReadCompressor<B>> {
        BufReadCompressor::new(self.device, self.pref, self.dict)
    }
}
