//! Streaming Compressors
mod async_bufread;
mod async_read;
mod async_write;
mod bufread;
mod read;
mod write;

use crate::Result;
pub use {bufread::*, read::*, write::*};

pub(crate) use super::api::CompressionContext;
pub(crate) use crate::lz4f::{Dictionary, Preferences};
pub(crate) use crate::Error;

#[cfg(feature = "tokio-io")]
pub use {async_bufread::*, async_read::*, async_write::*};

const LZ4F_HEADER_SIZE_MAX: usize = 19;

pub(crate) struct Compressor<D> {
    device: D,
    ctx: CompressionContext,
    pref: Preferences,
    state: State,
}

impl<D> Compressor<D> {
    pub fn new(device: D, pref: Preferences, dict: Option<Dictionary>) -> Result<Self> {
        Ok(Self {
            device,
            ctx: CompressionContext::new(dict)?,
            pref,
            state: State::Created,
        })
    }

    pub fn compress_bound(&self, src_size: usize) -> usize {
        CompressionContext::compress_bound(src_size, &self.pref)
    }
}

pub(crate) enum State {
    Created,
    WriteActive,
}
