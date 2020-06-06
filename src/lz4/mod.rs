mod api;
mod binding;

use crate::{LZ4Error, Result};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CompressionMode {
    Default,
    Fast(i32),
    FastExtState(i32),
}

impl Default for CompressionMode {
    fn default() -> Self {
        Self::Default
    }
}

pub fn compress(src: &[u8], dst: &mut [u8], mode: CompressionMode) -> Result<usize> {
    let len = match mode {
        CompressionMode::Default => api::compress_default(src, dst),
        CompressionMode::Fast(acc) => api::compress_fast(src, dst, acc),
        CompressionMode::FastExtState(acc) => {
            let mut state = vec![0; api::size_of_state()];
            api::compress_fast_ext_state(&mut state, src, dst, acc)
        }
    };
    if len > 0 {
        Ok(len)
    } else {
        Err(LZ4Error::from("Failed"))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DecompressionMode<'a> {
    Default,
    Partial,
    Dictionary(&'a [u8]),
}

impl<'a> Default for DecompressionMode<'a> {
    fn default() -> Self {
        Self::Default
    }
}
