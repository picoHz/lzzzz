#![allow(unsafe_code)]

use super::binding;

use libc::{c_char, c_int, c_uint, c_void, size_t};

pub fn size_of_state() -> usize {
    unsafe { binding::LZ4_sizeofStateHC() as usize }
}
