//! LZ4 Streaming Compressor/Decompressor

mod api;

use crate::{lz4::CompressionMode, Result};
use api::CompressionContext;

pub struct StreamCompressor<'a> {
    ctx: CompressionContext,
    dict: &'a [u8],
}

impl<'a> StreamCompressor<'a> {
    pub fn new() -> Result<Self> {
        Self::with_dict(&[])
    }

    pub fn with_dict(dict: &'a [u8]) -> Result<Self> {
        CompressionContext::new().map(|mut ctx| {
            ctx.set_dict(dict);
            Self { ctx, dict: &[] }
        })
    }

    pub fn next(&mut self, src: &'a [u8], dst: &mut [u8], mode: &CompressionMode) -> Result<()> {
        Ok(())
    }

    pub fn next_to_vec(
        &mut self,
        src: &'a [u8],
        dst: &mut Vec<u8>,
        mode: &CompressionMode,
    ) -> Result<()> {
        Ok(())
    }

    pub fn next_copy(&mut self, src: &[u8], dst: &mut [u8], mode: &CompressionMode) -> Result<()> {
        Ok(())
    }

    pub fn next_copy_to_vec(
        &mut self,
        src: &[u8],
        dst: &mut Vec<u8>,
        mode: &CompressionMode,
    ) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn compression_context() {
        super::StreamCompressor::new();
    }
}
