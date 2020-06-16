//! LZ4 Frame Streaming Compressor/Decompressor

mod api;
pub mod compressor;
pub mod decompressor;

use crate::{lz4f::Preferences, Result};
pub(crate) use api::DecompressionContext;
use api::DictionaryHandle;
use std::{convert::TryInto, sync::Arc};

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

/// A pre-compiled dictionary for the efficient compression.
///
/// **Cited from lz4frame.h:**
///
/// A Dictionary is useful for the compression of small messages (KB range).
/// It dramatically improves compression efficiency.
///
/// LZ4 can ingest any input as dictionary, though only the last 64 KB are
/// useful. Best results are generally achieved by using Zstandard's Dictionary
/// Builder to generate a high-quality dictionary from a set of samples.
#[derive(Clone)]
pub struct Dictionary(Arc<DictionaryHandle>);

impl Dictionary {
    pub fn new(data: &[u8]) -> Result<Self> {
        DictionaryHandle::new(data).map(|dict| Self(Arc::new(dict)))
    }
}
