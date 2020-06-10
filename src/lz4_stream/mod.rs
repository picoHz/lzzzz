//! LZ4 Streaming Compressor/Decompressor

mod api;

use crate::Result;
use std::{any::Any, ops::Deref, pin::Pin, rc::Rc};

pub struct StreamCompressor {
    outlives: Vec<Box<dyn Any>>,
}

impl StreamCompressor {
    pub fn new() -> Self {
        Self {
            outlives: Vec::new(),
        }
    }

    pub fn next_pin<P, T>(&mut self, src: Pin<P>, dst: &mut [u8]) -> Result<()>
    where
        P: 'static + Deref<Target = T>,
        T: ?Sized + AsRef<[u8]>,
    {
        self.outlives.push(Box::new(src));
        Ok(())
    }

    pub fn next_pin_to_vec<P, T>(&mut self, src: Pin<P>, dst: &mut Vec<u8>) -> Result<()>
    where
        P: 'static + Deref<Target = T>,
        T: ?Sized + AsRef<[u8]>,
    {
        self.outlives.push(Box::new(src));
        Ok(())
    }

    pub fn next(&mut self, src: &[u8], dst: &mut [u8]) -> Result<()> {
        Ok(())
    }

    pub fn next_to_vec(&mut self, src: &[u8], dst: &mut Vec<u8>) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn compression_context() {
        let mut a = super::StreamCompressor::new();
        let c = vec![0, 4];
        let r = { a.begin(&c, &mut []) };
        let c = vec![0, 4];
        let r = a.next(r, &c, &mut []);
    }
}
