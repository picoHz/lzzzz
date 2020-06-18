#![allow(unsafe_code)]

use super::super::binding;
use crate::{Error, LZ4Error, Report, Result};

use std::{
    cell::RefCell,
    ops::Deref,
    os::raw::{c_char, c_int, c_void},
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

pub fn decompress_safe(src: &[u8], dst: &mut [u8]) -> Result<Report> {
    let result = unsafe {
        binding::LZ4_decompress_safe(
            src.as_ptr() as *const c_char,
            dst.as_mut_ptr() as *mut c_char,
            src.len() as c_int,
            dst.len() as c_int,
        ) as i32
    };
    if result < 0 {
        Err(Error::LZ4Error(LZ4Error::Generic))
    } else {
        Ok(Report {
            dst_len: result as usize,
            ..Default::default()
        })
    }
}

pub fn decompress_safe_partial(src: &[u8], dst: &mut [u8], original_size: usize) -> Result<Report> {
    let result = unsafe {
        binding::LZ4_decompress_safe_partial(
            src.as_ptr() as *const c_char,
            dst.as_mut_ptr() as *mut c_char,
            src.len() as c_int,
            original_size as c_int,
            dst.len() as c_int,
        ) as i32
    };
    if result < 0 {
        Err(Error::LZ4Error(LZ4Error::Generic))
    } else {
        Ok(Report {
            dst_len: result as usize,
            ..Default::default()
        })
    }
}

pub fn decompress_safe_using_dict(src: &[u8], dst: &mut [u8], dict: &[u8]) -> Result<Report> {
    let result = unsafe {
        binding::LZ4_decompress_safe_usingDict(
            src.as_ptr() as *const c_char,
            dst.as_mut_ptr() as *mut c_char,
            src.len() as c_int,
            dst.len() as c_int,
            dict.as_ptr() as *const c_char,
            dict.len() as c_int,
        ) as i32
    };
    if result < 0 {
        Err(Error::LZ4Error(LZ4Error::Generic))
    } else {
        Ok(Report {
            dst_len: result as usize,
            ..Default::default()
        })
    }
}

#[derive(Clone)]
pub struct ExtState(RefCell<Box<[u8]>>);

impl ExtState {
    fn new() -> Self {
        let size = size_of_state() + 1;
        let mut buf = Vec::with_capacity(size);
        unsafe { buf.set_len(size) };
        buf[size - 1] = 0;
        Self(RefCell::new(buf.into_boxed_slice()))
    }

    pub fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&Self, bool) -> R,
    {
        #[cfg(feature = "use-tls")]
        {
            EXT_STATE.with(|state| {
                let reset = {
                    let mut state = state.borrow_mut();
                    let last = state.len() - 1;
                    if state[last] == 0 {
                        state[last] = 1;
                        false
                    } else {
                        true
                    }
                };

                (f)(state, reset)
            })
        }

        #[cfg(not(feature = "use-tls"))]
        {
            (f)(&Self::new(), false)
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
