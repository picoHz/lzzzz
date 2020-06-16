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

pub(crate) use super::api::DecompressionContext;
use crate::{lz4f::FrameInfo, Report, Result};
use std::convert::TryInto;

const LZ4F_HEADER_SIZE_MAX: usize = 19;

pub struct DecompressorBuilder<D> {
    device: D,
}

impl<D> DecompressorBuilder<D> {
    pub fn new(device: D) -> Self {
        Self { device }
    }

    pub fn build<T>(self) -> Result<T>
    where
        Self: TryInto<T, Error = crate::Error>,
    {
        self.try_into()
    }
}

pub(crate) struct Decompressor {
    ctx: DecompressionContext,
    header: [u8; LZ4F_HEADER_SIZE_MAX + 1],
    buffer: Vec<u8>,
}

impl Decompressor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            ctx: DecompressionContext::new()?,
            header: [0; LZ4F_HEADER_SIZE_MAX + 1],
            buffer: Vec::new(),
        })
    }

    pub fn get_frame_info(&mut self) -> Result<FrameInfo> {
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

        let len = self.buffer.len();
        self.buffer.reserve(1024);
        #[allow(unsafe_code)]
        unsafe {
            self.buffer.set_len(self.buffer.capacity());
        }
        let report = self.ctx.decompress(src, &mut self.buffer[len..], false)?;
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
