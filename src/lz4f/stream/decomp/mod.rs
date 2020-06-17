//! Streaming Decompressors
mod async_bufread;
mod async_read;
mod async_write;
mod bufread;
mod read;
mod write;

pub use bufread::*;
pub use read::*;
pub use write::*;

#[cfg(feature = "tokio-io")]
pub use {async_bufread::*, async_read::*, async_write::*};

use crate::{
    lz4f::{
        api::{DecompressionContext, LZ4F_HEADER_SIZE_MAX},
        FrameInfo,
    },
    Error, Report, Result,
};
use std::borrow::Cow;

pub(crate) struct Decompressor<'a> {
    ctx: DecompressionContext,
    header: [u8; LZ4F_HEADER_SIZE_MAX + 1],
    buffer: Vec<u8>,
    dict: Cow<'a, [u8]>,
    comp_dict: Option<*const u8>,
}

impl<'a> Decompressor<'a> {
    pub fn new() -> Result<Self> {
        Ok(Self {
            ctx: DecompressionContext::new()?,
            header: [0; LZ4F_HEADER_SIZE_MAX + 1],
            buffer: Vec::new(),
            dict: Cow::Borrowed(&[]),
            comp_dict: None,
        })
    }

    pub fn set_dict(&mut self, dict: Cow<'a, [u8]>) {
        self.dict = dict;
    }

    pub fn get_frame_info(&self) -> Result<FrameInfo> {
        let header_len = self.header[0] as usize;
        let (frame, _) = self.ctx.get_frame_info(&self.header[1..][..header_len])?;
        Ok(frame)
    }

    pub fn decompress(&mut self, src: &[u8]) -> Result<Report> {
        let header_len = self.header[0] as usize;
        if header_len < LZ4F_HEADER_SIZE_MAX {
            let len = std::cmp::min(LZ4F_HEADER_SIZE_MAX - header_len, src.len());
            (&mut self.header[1..][..len]).copy_from_slice(&src[..len]);
            self.header[0] += len as u8;
        }

        let dict_ptr = if self.dict.is_empty() {
            self.dict.as_ptr()
        } else {
            std::ptr::null()
        };
        if self.dict.as_ptr() != *self.comp_dict.get_or_insert(dict_ptr) {
            return Err(Error::DictionaryChangedDuringDecompression.into());
        }

        let len = self.buffer.len();
        self.buffer.reserve(1024);
        #[allow(unsafe_code)]
        unsafe {
            self.buffer.set_len(self.buffer.capacity());
        }
        let report = self
            .ctx
            .decompress_dict(src, &mut self.buffer[len..], &self.dict, false)?;
        self.buffer
            .resize_with(len + report.dst_len(), Default::default);
        Ok(report)
    }

    pub fn buf(&self) -> &[u8] {
        &self.buffer
    }

    pub fn clear_buf(&mut self) {
        self.buffer.clear();
    }
}
