#![allow(unsafe_code)]

use super::super::{binding, binding::LZ4StreamHC};
use crate::{Error, Report, Result};

use std::{
    os::raw::{c_char, c_int},
    ptr::NonNull,
};

pub struct CompressionContext {
    stream: NonNull<LZ4StreamHC>,
}

impl CompressionContext {
    pub fn new() -> Result<Self> {
        let ptr = unsafe { NonNull::new(binding::LZ4_createStreamHC()) };
        ptr.ok_or(Error::NullPointerUnexpected)
            .map(|stream| Self { stream })
    }

    #[cfg(feature = "liblz4-experimental")]
    pub fn set_compression_level(&mut self, compression_level: i32) {
        unsafe {
            binding::LZ4_setCompressionLevel(self.stream.as_ptr(), compression_level as c_int)
        }
    }

    #[cfg(feature = "liblz4-experimental")]
    pub fn set_favor_dec_speed(&mut self, flag: bool) {
        unsafe {
            binding::LZ4_favorDecompressionSpeed(
                self.stream.as_ptr(),
                if flag { 1 } else { 0 } as c_int,
            )
        }
    }

    pub fn reset(&mut self, compression_level: i32) {
        unsafe {
            binding::LZ4_resetStreamHC_fast(self.stream.as_ptr(), compression_level);
        }
    }

    pub fn load_dict(&mut self, dict: &[u8]) {
        unsafe {
            binding::LZ4_loadDictHC(
                self.stream.as_ptr(),
                dict.as_ptr() as *const c_char,
                dict.len() as c_int,
            );
        }
    }

    pub fn next(&mut self, src: &[u8], dst: &mut [u8]) -> Result<Report> {
        let dst_len = unsafe {
            binding::LZ4_compress_HC_continue(
                self.stream.as_ptr(),
                src.as_ptr() as *const c_char,
                dst.as_mut_ptr() as *mut c_char,
                src.len() as c_int,
                dst.len() as c_int,
            ) as usize
        };
        if dst_len > 0 {
            Ok(Report {
                dst_len,
                ..Default::default()
            })
        } else if src.is_empty() && dst.is_empty() {
            Ok(Report::default())
        } else {
            Err(Error::CompressionFailed)
        }
    }

    pub fn next_partial(&mut self, src: &[u8], dst: &mut [u8]) -> Result<Report> {
        let mut src_len = src.len() as c_int;
        let dst_len = unsafe {
            binding::LZ4_compress_HC_continue_destSize(
                self.stream.as_ptr(),
                src.as_ptr() as *const c_char,
                dst.as_mut_ptr() as *mut c_char,
                &mut src_len as *mut c_int,
                dst.len() as c_int,
            ) as usize
        };
        if dst_len > 0 {
            Ok(Report {
                dst_len,
                src_len: Some(src_len as usize),
            })
        } else if src.is_empty() && dst.is_empty() {
            Ok(Report::default())
        } else {
            Err(Error::CompressionFailed)
        }
    }
}

impl Drop for CompressionContext {
    fn drop(&mut self) {
        unsafe {
            binding::LZ4_freeStreamHC(self.stream.as_mut());
        }
    }
}
