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
    ExtState(ExtSate),
    DestSize(ExtSate),
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

pub fn compress(
    src: &[u8],
    dst: &mut [u8],
    mode: CompressionMode,
    compression_level: CompressionLevel,
) -> Result<usize> {
    todo!();
}

pub fn compress_to_vec(
    src: &[u8],
    dst: &mut Vec<u8>,
    mode: CompressionMode,
    compression_level: CompressionLevel,
) -> Result<()> {
    todo!();
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ExtSate(Rc<RefCell<Option<Box<[u8]>>>>);

impl ExtSate {
    pub fn new() -> Self {
        Default::default()
    }

    pub(crate) fn borrow_mut(&self) -> RefMut<'_, Box<[u8]>> {
        let mut data = self.0.borrow_mut();
        if data.is_none() {
            data.replace(vec![0; api::size_of_state()].into_boxed_slice());
        }
        RefMut::map(data, |data| data.as_mut().unwrap())
    }
}
