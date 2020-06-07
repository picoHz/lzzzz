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

pub fn compress(src: &[u8], dst: &mut [u8], mode: CompressionMode) -> Result<usize> {
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
