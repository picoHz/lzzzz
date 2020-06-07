mod api;
mod binding;

use api::DictionaryHandle;
use std::{cmp, io, ops, sync::Arc};

/// A user-defined dictionary for the efficient compression.
///
/// **Cited from lz4frame.h:**
///
/// A Dictionary is useful for the compression of small messages (KB range).
/// It dramatically improves compression efficiency.
///
/// LZ4 can ingest any input as dictionary, though only the last 64 KB are useful.
/// Best results are generally achieved by using Zstandard's Dictionary Builder
/// to generate a high-quality dictionary from a set of samples.
#[derive(Clone)]
pub struct Dictionary(Arc<DictionaryHandle>);

impl Dictionary {
    pub fn new(data: &[u8]) -> Self {
        Self(Arc::new(DictionaryHandle::new(data)))
    }
}
