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
) -> Report {
    let dst_len = unsafe {
        binding::LZ4_compress_fast_extState(
            state.as_mut_ptr() as *mut c_void,
            src.as_ptr() as *const c_char,
            dst.as_mut_ptr() as *mut c_char,
            src.len() as c_int,
            dst.len() as c_int,
            acceleration as c_int,
        ) as usize
    };
    Report {
        dst_len,
        ..Default::default()
    }
}

pub fn compress_fast_ext_state_fast_reset(
    state: &mut [u8],
    src: &[u8],
    dst: &mut [u8],
    acceleration: i32,
) -> Report {
    let dst_len = unsafe {
        binding::LZ4_compress_fast_extState_fastReset(
            state.as_mut_ptr() as *mut c_void,
            src.as_ptr() as *const c_char,
            dst.as_mut_ptr() as *mut c_char,
            src.len() as c_int,
            dst.len() as c_int,
            acceleration as c_int,
        ) as usize
    };
    Report {
        dst_len,
        ..Default::default()
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

pub fn decompress_safe_using_dict(src: &[u8], dst: &mut [u8], dict: &[u8]) -> Report {
    let dst_len = unsafe {
        binding::LZ4_decompress_safe_usingDict(
            src.as_ptr() as *const c_char,
            dst.as_mut_ptr() as *mut c_char,
            src.len() as c_int,
            dst.len() as c_int,
            dict.as_ptr() as *const c_char,
            dict.len() as c_int,
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
    fn new() -> Self {
        let size = size_of_state() + 1;
        let mut buf = Vec::with_capacity(size);
        unsafe { buf.set_len(size) };
        *buf.last_mut().unwrap() = 0;
        Self(RefCell::new(buf.into_boxed_slice()))
    }

    pub fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&Self) -> R,
    {
        #[cfg(feature = "use-tls")]
        {
            EXT_STATE.with(f)
        }

        #[cfg(not(feature = "use-tls"))]
        {
            (f)(&Self::new())
        }
    }
}

impl Deref for ExtState {
    type Target = RefCell<Box<[u8]>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "use-tls")]
thread_local!(static EXT_STATE: ExtState = ExtState::new());
