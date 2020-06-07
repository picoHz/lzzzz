#![allow(unsafe_code)]

use super::binding;

use libc::{c_char, c_int, c_void};
use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

pub fn compress(src: &[u8], dst: &mut [u8], compression_level: i32) -> usize {
    unsafe {
        binding::LZ4_compress_HC(
            src.as_ptr() as *const c_char,
            dst.as_mut_ptr() as *mut c_char,
            src.len() as c_int,
            dst.len() as c_int,
            compression_level as c_int,
        ) as usize
    }
}

pub fn compress_ext_state(
    state: &mut [u8],
    src: &[u8],
    dst: &mut [u8],
    compression_level: i32,
) -> usize {
    unsafe {
        binding::LZ4_compress_HC_extStateHC(
            state.as_mut_ptr() as *mut c_void,
            src.as_ptr() as *const c_char,
            dst.as_mut_ptr() as *mut c_char,
            src.len() as c_int,
            dst.len() as c_int,
            compression_level as c_int,
        ) as usize
    }
}

pub fn compress_dest_size(
    state: &mut [u8],
    src: &[u8],
    dst: &mut [u8],
    compression_level: i32,
) -> usize {
    let mut src_size = src.len() as i32;
    unsafe {
        binding::LZ4_compress_HC_destSize(
            state.as_mut_ptr() as *mut c_void,
            src.as_ptr() as *const c_char,
            dst.as_mut_ptr() as *mut c_char,
            &mut src_size as *mut c_int,
            dst.len() as c_int,
            compression_level as c_int,
        ) as usize
    }
}

pub fn size_of_state() -> usize {
    unsafe { binding::LZ4_sizeofStateHC() as usize }
}

#[derive(Clone)]
pub struct ExtState(Rc<RefCell<Box<[u8]>>>);

impl ExtState {
    pub fn new() -> Self {
        let size = size_of_state();
        let mut buf = Vec::with_capacity(size);
        unsafe { buf.set_len(size) };
        Self(Rc::new(RefCell::new(buf.into_boxed_slice())))
    }

    pub fn get() -> Self {
        EXT_STATE.with(Clone::clone)
    }

    pub fn borrow_mut(&self) -> RefMut<'_, Box<[u8]>> {
        self.borrow_mut()
    }
}

thread_local!(static EXT_STATE: ExtState = ExtState::new());
