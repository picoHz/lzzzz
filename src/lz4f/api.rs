#![allow(unsafe_code)]

use super::Preferences;
use crate::{binding, LZ4Error, Report, Result};
use libc::{c_char, c_int, c_void, size_t};
use std::ffi::CStr;

pub fn compress_bound(input_size: usize, prefs: &Preferences) -> usize {
    unsafe {
        binding::LZ4F_compressBound(input_size as size_t, prefs as *const Preferences) as usize
    }
}

pub fn compress(src: &[u8], dst: &mut [u8], prefs: &Preferences) -> Result<Report> {
    let code = unsafe {
        binding::LZ4F_compressFrame(
            dst.as_mut_ptr() as *mut c_void,
            dst.len() as size_t,
            src.as_ptr() as *const c_void,
            src.len() as size_t,
            prefs as *const Preferences,
        ) as usize
    };
    make_result(
        Report {
            dst_len: code,
            ..Default::default()
        },
        code,
    )
}

fn make_result<T>(data: T, code: size_t) -> Result<T> {
    unsafe {
        if binding::LZ4F_isError(code) != 0 {
            Err(LZ4Error::from(
                CStr::from_ptr(binding::LZ4F_getErrorName(code))
                    .to_str()
                    .unwrap(),
            ))
        } else {
            Ok(data)
        }
    }
}
