mod api;
mod binding;

use api::{Preferences, CompressionContext};
use std::io;
use crate::Result;

pub struct CompressorBuilder {
    prefs: Preferences,
}

impl CompressorBuilder {
    pub fn new() -> Self {
        Self {
            prefs: Default::default(),
        }
    }

    pub fn build<'a, D>(self, io: &'a mut D) -> Result<Compressor<'a, D>> {
        Ok(Compressor {
            prefs: self.prefs,
            ctx: CompressionContext::new()?,
            io,
        })
    }
}

pub struct Compressor<'a, D> {
    prefs: Preferences,
    ctx: CompressionContext,
    io: &'a mut D,
}

impl<'a, D: io::Write> io::Write for Compressor<'a, D> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.io.write(&[])
    }
    fn flush(&mut self) -> io::Result<()> { 
        self.io.flush()
    }
}

impl<'a, D> Drop for Compressor<'a, D> {
    fn drop(&mut self) { 
        todo!() 
    }
}