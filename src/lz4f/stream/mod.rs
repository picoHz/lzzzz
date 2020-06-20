//! LZ4 Frame Streaming Compressor/Decompressor

pub mod comp;
pub mod decomp;

use crate::common::DEFAULT_BUF_SIZE;
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

    /// Set the compression preferences.
    pub const fn preferences(mut self, pref: Preferences) -> Self {
        self.pref = pref;
        self
    }

    /// Set the compression dictionary.
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
    capacity: usize,
}

impl<D> DecompressorBuilder<D> {
    pub const fn new(device: D) -> Self {
        Self {
            device,
            capacity: DEFAULT_BUF_SIZE,
        }
    }

    /// Set the capacity of the internal buffer.
    pub const fn capacity(mut self, capacity: usize) -> Self {
        self.capacity = capacity;
        self
    }

    pub fn build<T>(self) -> Result<T>
    where
        Self: TryInto<T, Error = crate::Error>,
    {
        self.try_into()
    }
}
