mod api;

use crate::{common::DICTIONARY_SIZE, lz4, lz4_hc::FavorDecSpeed, Result};
use api::CompressionContext;
use std::{borrow::Cow, pin::Pin};

/// Streaming LZ4_HC compressor.
pub struct Compressor<'a> {
    ctx: CompressionContext,
    dict: Pin<Cow<'a, [u8]>>,
    safe_buf: Vec<u8>,
}

impl<'a> Compressor<'a> {
    pub fn new() -> Result<Self> {
        Ok(Self {
            ctx: CompressionContext::new()?,
            dict: Pin::new(Cow::Borrowed(&[])),
            safe_buf: Vec::new(),
        })
    }

    pub fn with_dict<D>(dict: D) -> Result<Self>
    where
        D: Into<Cow<'a, [u8]>>,
    {
        let mut comp = Self {
            dict: Pin::new(dict.into()),
            ..Self::new()?
        };
        comp.ctx.load_dict(&comp.dict);
        Ok(comp)
    }

    pub fn set_compression_level(&mut self, level: i32) {
        self.ctx.set_compression_level(level);
    }

    pub fn set_favor_dec_speed(&mut self, dec_speed: FavorDecSpeed) {
        self.ctx
            .set_favor_dec_speed(dec_speed == FavorDecSpeed::Enabled);
    }

    pub fn next(&mut self, src: &[u8], dst: &mut [u8]) -> Result<usize> {
        let result = self.ctx.next(&src, dst)?;
        self.save_dict();
        Ok(result)
    }

    pub fn next_partial(&mut self, src: &[u8], dst: &mut [u8]) -> Result<(usize, usize)> {
        let result = self.ctx.next_partial(&src, dst)?;
        self.save_dict();
        Ok(result)
    }

    pub fn next_to_vec(&mut self, src: &[u8], dst: &mut Vec<u8>) -> Result<usize> {
        let orig_len = dst.len();
        dst.reserve(lz4::max_compressed_size(src.len()));
        #[allow(unsafe_code)]
        unsafe {
            dst.set_len(dst.capacity());
        }
        let result = self.next(src, &mut dst[orig_len..]);
        dst.resize_with(orig_len + result.as_ref().unwrap_or(&0), Default::default);
        result
    }

    fn save_dict(&mut self) {
        self.safe_buf.resize(DICTIONARY_SIZE, 0);
        self.ctx.save_dict(&mut self.safe_buf);
    }
}
