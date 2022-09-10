use super::api::DictionaryHandle;
use std::sync::Arc;

/// A pre-compiled dictionary for the efficient compression.
#[derive(Clone)]
pub struct Dictionary(Arc<DictionaryHandle>);

impl Dictionary {
    /// Builds a new `Dictionary`.
    #[cfg(not(feature = "system-liblz4"))]
    pub fn new(data: &[u8]) -> super::Result<Self> {
        Ok(Self(Arc::new(DictionaryHandle::new(data)?)))
    }

    #[cfg(not(feature = "system-liblz4"))]
    pub(crate) fn handle(&self) -> &DictionaryHandle {
        &self.0
    }
}

#[cfg(test)]
#[cfg(not(feature = "system-liblz4"))]
mod tests {
    use super::Dictionary;

    #[test]
    fn create_dictionary() {
        assert!(Dictionary::new(&[]).is_ok());
        assert!(Dictionary::new(&b"quick brown fox jumps over the lazy dog"[..]).is_ok());
    }
}
