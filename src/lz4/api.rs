#![allow(unsafe_code)]

use super::binding;

use libc::{c_char, c_int, c_void};
use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

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

pub fn decompress_safe(src: &[u8], dst: &mut [u8]) -> i32 {
    unsafe {
        binding::LZ4_decompress_safe(
            src.as_ptr() as *const c_char,
            dst.as_mut_ptr() as *mut c_char,
            src.len() as c_int,
            dst.len() as c_int,
        ) as i32
    }
}

#[derive(Clone)]
pub struct ExtState(Rc<RefCell<Box<[u8]>>>);

impl ExtState {
    fn new() -> Self {
        let size = size_of_state();
        let mut buf = Vec::with_capacity(size);
        unsafe { buf.set_len(size) };
        Self(Rc::new(RefCell::new(buf.into_boxed_slice())))
    }

    pub fn get() -> Self {
        EXT_STATE.with(Clone::clone)
    }

    pub fn borrow_mut(&self) -> RefMut<'_, Box<[u8]>> {
        RefCell::borrow_mut(&self.0)
    }
}

thread_local!(static EXT_STATE: ExtState = ExtState::new());
