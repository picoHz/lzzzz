//! LZ4 Frame Streaming Compressor/Decompressor

pub mod comp;
pub mod decomp;

use crate::{
    lz4f::{Dictionary, Preferences},
    Result,
};
use std::convert::TryInto;

/// A builder struct to create a streaming compressor
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

/// A builder struct to create a streaming decompressor
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
