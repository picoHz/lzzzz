use super::{api::DictionaryHandle, Result};
use std::sync::Arc;

/// A pre-compiled dictionary for the efficient compression
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
pub struct SharedDict(Arc<DictionaryHandle>);

impl SharedDict {
    /// Build a new `SharedDict`.
    pub fn new(data: &[u8]) -> Result<Self> {
        DictionaryHandle::new(data).map(|dict| Self(Arc::new(dict)))
    }

    pub(crate) fn handle(&self) -> &DictionaryHandle {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::SharedDict;
    use static_assertions::assert_impl_all;

    assert_impl_all!(SharedDict: Send, Sync);

    #[test]
    fn create_dictionary() {
        assert!(SharedDict::new(&[]).is_ok());
        assert!(SharedDict::new(&b"quick brown fox jumps over the lazy dog"[..]).is_ok());
    }
}
