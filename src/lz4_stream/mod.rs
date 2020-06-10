//! LZ4 Streaming Compressor/Decompressor

mod api;

use crate::{lz4::CompressionMode, Result};
use api::CompressionContext;
use std::borrow::Cow;

pub struct StreamCompressor<'a> {
    ctx: CompressionContext,
    dict: Cow<'a, [u8]>,
}

impl<'a> StreamCompressor<'a> {
    pub fn new() -> Result<Self> {
        Self::with_dict(Cow::Borrowed(&[]))
    }

    pub fn with_dict(dict: Cow<'a, [u8]>) -> Result<Self> {
        CompressionContext::new().map(|mut ctx| {
            ctx.set_dict(&dict);
            Self { ctx, dict }
        })
    }

    pub fn next(
        &mut self,
        src: Cow<'a, [u8]>,
        dst: &mut [u8],
        mode: &CompressionMode,
    ) -> Result<()> {
        Ok(())
    }

    pub fn next_to_vec(
        &mut self,
        src: Cow<'a, [u8]>,
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
        use crate::lz4::CompressionMode;
        use std::borrow::Cow;
        let mut cp = super::StreamCompressor::new().unwrap();
        cp.next(Cow::Owned(vec![]), &mut [], &CompressionMode::Default);
    }
}
