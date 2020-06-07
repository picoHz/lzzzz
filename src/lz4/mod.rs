//! LZ4 Compressor/Decompressor
mod api;
mod binding;

use crate::{LZ4Error, Result};
use std::{
    cell::{RefCell, RefMut},
    ops::Deref,
    rc::Rc,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompressionMode {
    Default,
    Fast(i32),
    FastExtState(i32, ExtSate),
}

impl Default for CompressionMode {
    fn default() -> Self {
        Self::Default
    }
}

pub fn max_compressed_size(src_size: usize) -> usize {
    api::compress_bound(src_size)
}

pub fn compress_to_slice(src: &[u8], dst: &mut [u8], mode: CompressionMode) -> Result<usize> {
    let len = match mode {
        CompressionMode::Default => api::compress_default(src, dst),
        CompressionMode::Fast(acc) => api::compress_fast(src, dst, acc),
        CompressionMode::FastExtState(acc, state) => {
            api::compress_fast_ext_state(&mut state.borrow_mut(), src, dst, acc)
        }
    };
    if len > 0 {
        Ok(len)
    } else {
        Err(LZ4Error::from("Compression failed"))
    }
}

pub fn compress(src: &[u8], dst: &mut Vec<u8>, mode: CompressionMode) -> Result<()> {
    dst.resize_with(max_compressed_size(src.len()), Default::default);
    let result = compress_to_slice(src, dst, mode);
    dst.resize_with(*result.as_ref().unwrap_or(&0), Default::default);
    result.map(|_| ())
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

pub fn decompress(src: &[u8], dst: &mut [u8], mode: DecompressionMode) -> Result<()> {
    todo!();
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ExtSate(Rc<RefCell<Option<Box<[u8]>>>>);

impl ExtSate {
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

thread_local!(static EXT_STATE: ExtSate = ExtSate::new());
