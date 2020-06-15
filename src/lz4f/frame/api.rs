#![allow(unsafe_code)]

use super::Preferences;
use crate::{binding, common, Report, Result};
use libc::{c_void, size_t};

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
    common::result_from_code(code).map(|_| Report {
        dst_len: code,
        ..Default::default()
    })
}
