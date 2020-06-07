//! LZ4_HC Compressor
//!
//! The `lz4_hc` module doesn't provide decompression functionarities.
//! Use the `lz4` module instead.
mod api;
mod binding;

use crate::Result;
use std::{
    cell::{RefCell, RefMut},
    ops::Deref,
    rc::Rc,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompressionMode {
    Default,
    ExtState(ExtState),
    DestSize(ExtState),
}

impl Default for CompressionMode {
    fn default() -> Self {
        Self::Default
    }
}

/// Compression level.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CompressionLevel {
    Custom(i32),
    Min,
    Default,
    OptMin,
    Max,
}

impl CompressionLevel {}

pub fn max_compressed_size(uncompressed_size: usize) -> usize {
    0
}

pub fn compress_to_slice(
    src: &[u8],
    dst: &mut [u8],
    mode: CompressionMode,
    compression_level: CompressionLevel,
) -> Result<usize> {
    todo!();
}

/// Read data from a slice and append a compressed data to `Vec<u8>`.
pub fn compress(
    src: &[u8],
    dst: &mut Vec<u8>,
    mode: CompressionMode,
    compression_level: CompressionLevel,
) -> Result<()> {
    todo!();
}

/// ExtState
///
/// To reduce allocation overhead, the `ExtState` is implemented as a shared buffer.
/// No matter how many times you call `ExtState::new()` or `ExtState::clone()`,
/// the heap allocation occurs only once per thread.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ExtState(Rc<RefCell<Option<Box<[u8]>>>>);

impl ExtState {
    pub fn new() -> Self {
        EXT_STATE.with(Clone::clone)
    }

    pub(crate) fn borrow_mut(&self) -> RefMut<'_, Box<[u8]>> {
        let mut data = self.0.borrow_mut();
        if data.is_none() {
            data.replace(vec![0; api::size_of_state()].into_boxed_slice());
        }
        RefMut::map(data, |data| data.as_mut().unwrap())
    }
}

thread_local!(static EXT_STATE: ExtState = Default::default());
