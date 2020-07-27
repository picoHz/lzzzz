//! Streaming LZ4F compressors.
mod async_bufread;
mod async_read;
mod async_write;
mod bufread;
mod read;
mod write;

use crate::lz4f::Result;

pub use bufread::*;
pub use read::*;
pub use write::*;

use crate::lz4f::{
    api::{CompressionContext, LZ4F_HEADER_SIZE_MAX},
    Dictionary, Preferences,
};

#[cfg(feature = "async-io")]
pub use {async_bufread::*, async_read::*, async_write::*};

pub(crate) struct Compressor {
    ctx: CompressionContext,
    prefs: Preferences,
    state: State,
    buffer: Vec<u8>,
}

impl Compressor {
    pub fn new(prefs: Preferences, dict: Option<Dictionary>) -> Result<Self> {
        Ok(Self {
            ctx: CompressionContext::new(dict)?,
            prefs,
            state: State::Created,
            buffer: Vec::new(),
        })
    }

    fn begin(&mut self) -> Result<()> {
        if let State::Created = self.state {
            assert!(self.buffer.is_empty());
            self.state = State::Active;
            self.buf_resize(LZ4F_HEADER_SIZE_MAX);
            let len = self.ctx.begin(&mut self.buffer, &self.prefs)?;
            self.buf_resize(len);
        }
        Ok(())
    }

    pub fn update(&mut self, src: &[u8], stable_src: bool) -> Result<()> {
        self.begin()?;
        let offset = self.buf_extend_bound(src.len());
        let len = self
            .ctx
            .update(&mut self.buffer[offset..], src, stable_src)?;
        self.buf_resize(offset + len);
        if len == 0 {
            self.flush(stable_src)
        } else {
            Ok(())
        }
    }

    pub fn flush(&mut self, stable_src: bool) -> Result<()> {
        self.begin()?;
        let offset = self.buf_extend_bound(0);
        let len = self.ctx.flush(&mut self.buffer[offset..], stable_src)?;
        self.buf_resize(offset + len);
        Ok(())
    }

    pub fn end(&mut self, stable_src: bool) -> Result<()> {
        self.begin()?;
        if let State::Active = self.state {
            self.state = State::Finished;
            let offset = self.buf_extend_bound(0);
            let len = self.ctx.end(&mut self.buffer[offset..], stable_src)?;
            self.buf_resize(offset + len);
        }
        Ok(())
    }

    pub fn buf(&self) -> &[u8] {
        &self.buffer
    }

    pub fn clear_buf(&mut self) {
        self.buffer.clear();
    }

    fn buf_resize(&mut self, size: usize) {
        if self.buffer.capacity() < size {
            self.buffer.reserve(size - self.buffer.len());
        }
        #[allow(unsafe_code)]
        unsafe {
            self.buffer.set_len(size);
        }
    }

    fn buf_extend_bound(&mut self, src_size: usize) -> usize {
        let len = CompressionContext::compress_bound(src_size, &self.prefs);
        let old_len = self.buffer.len();
        self.buf_resize(old_len + len);
        old_len
    }
}

pub(crate) enum State {
    Created,
    Active,
    Finished,
}
