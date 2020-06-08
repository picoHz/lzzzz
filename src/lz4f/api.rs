#![allow(unsafe_code)]

use super::Preferences;
use crate::{binding, LZ4Error, Report, Result};
use binding::{LZ4FDecompressionCtx, LZ4FDecompressionOptions};
use libc::{c_char, c_int, c_void, size_t};
use std::{ffi::CStr, ptr::NonNull};

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

pub struct DecompressionContext {
    ctx: NonNull<LZ4FDecompressionCtx>,
}

impl DecompressionContext {
    pub fn new() -> Result<Self> {
        let mut ctx: *mut LZ4FDecompressionCtx = std::ptr::null_mut();
        let code = unsafe {
            binding::LZ4F_createDecompressionContext(
                &mut ctx as *mut *mut binding::LZ4FDecompressionCtx,
                binding::LZ4F_getVersion(),
            )
        };
        make_result(
            Self {
                ctx: NonNull::new(ctx).unwrap(),
            },
            code,
        )
    }

    pub fn decompress(
        &mut self,
        src: &[u8],
        dst: &mut [u8],
        opt: Option<&LZ4FDecompressionOptions>,
    ) -> Result<Report> {
        let mut dst_len = dst.len() as size_t;
        let mut src_len = src.len() as size_t;
        let code = unsafe {
            binding::LZ4F_decompress(
                self.ctx.as_ptr(),
                dst.as_mut_ptr() as *mut c_void,
                &mut dst_len as *mut size_t,
                src.as_ptr() as *const c_void,
                &mut src_len as *mut size_t,
                opt.map(|p| p as *const LZ4FDecompressionOptions)
                    .unwrap_or(std::ptr::null()),
            )
        };
        make_result(
            Report {
                src_len: Some(src_len as usize),
                dst_len: dst_len as usize,
            },
            code,
        )
    }
}

impl Drop for DecompressionContext {
    fn drop(&mut self) {
        unsafe {
            binding::LZ4F_freeDecompressionContext(self.ctx.as_ptr());
        }
    }
}
