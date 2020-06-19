#![allow(unsafe_code)]

use super::super::binding;
use crate::Report;

use std::{
    cell::RefCell,
    ops::Deref,
    os::raw::{c_char, c_int, c_void},
};

pub const fn size_of_state() -> usize {
    binding::LZ4_STREAMHCSIZE
}

pub fn compress_ext_state(
    state: &mut [u8],
    src: &[u8],
    dst: &mut [u8],
    compression_level: i32,
) -> Report {
    let dst_len = unsafe {
        binding::LZ4_compress_HC_extStateHC(
            state.as_mut_ptr() as *mut c_void,
            src.as_ptr() as *const c_char,
            dst.as_mut_ptr() as *mut c_char,
            src.len() as c_int,
            dst.len() as c_int,
            compression_level as c_int,
        ) as usize
    };
    Report {
        dst_len,
        ..Default::default()
    }
}

pub fn compress_ext_state_fast_reset(
    state: &mut [u8],
    src: &[u8],
    dst: &mut [u8],
    compression_level: i32,
) -> Report {
    let dst_len = unsafe {
        binding::LZ4_compress_HC_extStateHC_fastReset(
            state.as_mut_ptr() as *mut c_void,
            src.as_ptr() as *const c_char,
            dst.as_mut_ptr() as *mut c_char,
            src.len() as c_int,
            dst.len() as c_int,
            compression_level as c_int,
        ) as usize
    };
    Report {
        dst_len,
        ..Default::default()
    }
}

pub fn compress_dest_size(
    state: &mut [u8],
    src: &[u8],
    dst: &mut [u8],
    compression_level: i32,
) -> Report {
    let mut src_len = src.len() as i32;
    let dst_len = unsafe {
        binding::LZ4_compress_HC_destSize(
            state.as_mut_ptr() as *mut c_void,
            src.as_ptr() as *const c_char,
            dst.as_mut_ptr() as *mut c_char,
            &mut src_len as *mut c_int,
            dst.len() as c_int,
            compression_level as c_int,
        ) as usize
    };
    Report {
        src_len: Some(src_len as usize),
        dst_len,
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
