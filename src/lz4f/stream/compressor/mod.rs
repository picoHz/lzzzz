//! Streaming Compressors
mod async_bufread;
mod async_read;
mod async_write;
mod bufread;
mod read;
mod write;

use crate::Result;
use std::convert::TryInto;

pub use bufread::*;
pub use read::*;
pub use write::*;

pub(crate) use super::api::CompressionContext;
pub(crate) use crate::{
    lz4f::{Dictionary, Preferences},
    Error,
};

#[cfg(feature = "tokio-io")]
pub use {async_bufread::*, async_read::*, async_write::*};

const LZ4F_HEADER_SIZE_MAX: usize = 19;

pub struct CompressorBuilder<D> {
    device: D,
    pref: Preferences,
    dict: Option<Dictionary>,
}

impl<D> CompressorBuilder<D> {
    pub fn new(device: D) -> Self {
        Self {
            device,
            pref: Default::default(),
            dict: None,
        }
    }

    pub fn preferences(mut self, pref: Preferences) -> Self {
        self.pref = pref;
        self
    }

    pub fn dict(mut self, dict: Dictionary) -> Self {
        self.dict = Some(dict);
        self
    }

    pub fn build<T>(self) -> Result<T>
    where
        Self: TryInto<T, Error = crate::Error>,
    {
        self.try_into()
    }
}

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
