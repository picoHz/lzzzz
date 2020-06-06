#![allow(unsafe_code)]

use super::binding;

use libc::{c_char, c_int, c_uint, c_void, size_t};

pub fn compress_default(src: &[u8], dst: &mut [u8]) -> usize {
    unsafe {
        binding::LZ4_compress_default(
            src.as_ptr() as *const c_char,
            dst.as_mut_ptr() as *mut c_char,
            src.len() as c_int,
            dst.len() as c_int,
        ) as usize
    }
}

pub fn compress_fast(src: &[u8], dst: &mut [u8], acceleration: i32) -> usize {
    unsafe {
        binding::LZ4_compress_fast(
            src.as_ptr() as *const c_char,
            dst.as_mut_ptr() as *mut c_char,
            src.len() as c_int,
            dst.len() as c_int,
            acceleration as c_int,
        ) as usize
    }
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
