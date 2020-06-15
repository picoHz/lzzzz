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
use crate::Result;
use std::convert::TryInto;

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
}

impl Decompressor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            ctx: DecompressionContext::new()?,
        })
    }
}
