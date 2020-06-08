#![allow(unsafe_code)]

use crate::{binding, Report};

use libc::{c_char, c_int, c_void};
use std::{cell::RefCell, ops::Deref};

pub fn compress_bound(input_size: usize) -> usize {
    unsafe { binding::LZ4_compressBound(input_size as c_int) as usize }
}

pub fn size_of_state() -> usize {
    unsafe { binding::LZ4_sizeofState() as usize }
}

pub fn compress_fast_ext_state(
    state: &mut [u8],
    src: &[u8],
    dst: &mut [u8],
    acceleration: i32,
) -> usize {
    unsafe {
        binding::LZ4_compress_fast_extState(
            state.as_mut_ptr() as *mut c_void,
            src.as_ptr() as *const c_char,
            dst.as_mut_ptr() as *mut c_char,
            src.len() as c_int,
            dst.len() as c_int,
            acceleration as c_int,
        ) as usize
    }
}

pub fn decompress_safe(src: &[u8], dst: &mut [u8]) -> Report {
    let dst_len = unsafe {
        binding::LZ4_decompress_safe(
            src.as_ptr() as *const c_char,
            dst.as_mut_ptr() as *mut c_char,
            src.len() as c_int,
            dst.len() as c_int,
        ) as usize
    };
    Report {
        dst_len,
        ..Default::default()
    }
}

pub fn decompress_safe_partial(src: &[u8], dst: &mut [u8], original_size: usize) -> Report {
    let dst_len = unsafe {
        binding::LZ4_decompress_safe_partial(
            src.as_ptr() as *const c_char,
            dst.as_mut_ptr() as *mut c_char,
            src.len() as c_int,
            original_size as c_int,
            dst.len() as c_int,
        ) as usize
    };
    Report {
        dst_len,
        ..Default::default()
    }
}

#[derive(Clone)]
pub struct ExtState(RefCell<Box<[u8]>>);

impl ExtState {
    pub fn new() -> Self {
        let size = size_of_state();
        let mut buf = Vec::with_capacity(size);
        unsafe { buf.set_len(size) };
        Self(RefCell::new(buf.into_boxed_slice()))
    }
}

impl Deref for ExtState {
    type Target = RefCell<Box<[u8]>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
