#![allow(unsafe_code)]

use super::{FrameInfo, Preferences};
use crate::{binding, common, Error, Report, Result};
use binding::{LZ4FDecompressionCtx, LZ4FDecompressionOptions};
use libc::{c_char, c_int, c_void, size_t};
use std::{ffi::CStr, mem::MaybeUninit, ptr::NonNull};

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
        common::result_from_code(code).map(|_| Self {
            ctx: NonNull::new(ctx).unwrap(),
        })
    }

    pub fn get_frame_info(&mut self, src: &[u8]) -> Result<(FrameInfo, Report)> {
        let mut info = MaybeUninit::<FrameInfo>::uninit();
        let mut src_len = src.len() as size_t;
        let code = unsafe {
            binding::LZ4F_getFrameInfo(
                self.ctx.as_ptr(),
                info.as_mut_ptr() as *mut FrameInfo,
                src.as_ptr() as *const c_void,
                &mut src_len as *mut size_t,
            )
        };
        common::result_from_code(code).map(|_| {
            (
                unsafe { info.assume_init() },
                Report {
                    src_len: Some(src_len as usize),
                    dst_len: 0,
                    expected_len: Some(code as usize),
                },
            )
        })
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
        common::result_from_code(code).map(|_| Report {
            src_len: Some(src_len as usize),
            dst_len: dst_len as usize,
            expected_len: Some(code as usize),
        })
    }

    pub fn decompress_dict(
        &mut self,
        src: &[u8],
        dst: &mut [u8],
        dict: &[u8],
        opt: Option<&LZ4FDecompressionOptions>,
    ) -> Result<Report> {
        let mut dst_len = dst.len() as size_t;
        let mut src_len = src.len() as size_t;
        let code = unsafe {
            binding::LZ4F_decompress_usingDict(
                self.ctx.as_ptr(),
                dst.as_mut_ptr() as *mut c_void,
                &mut dst_len as *mut size_t,
                src.as_ptr() as *const c_void,
                &mut src_len as *mut size_t,
                dict.as_ptr() as *const c_void,
                dict.len() as size_t,
                opt.map(|p| p as *const LZ4FDecompressionOptions)
                    .unwrap_or(std::ptr::null()),
            )
        };
        common::result_from_code(code).map(|_| Report {
            src_len: Some(src_len as usize),
            dst_len: dst_len as usize,
            expected_len: Some(code as usize),
        })
    }

    pub fn reset(&mut self) {
        unsafe {
            binding::LZ4F_resetDecompressionContext(self.ctx.as_ptr());
        }
    }
}

impl Drop for DecompressionContext {
    fn drop(&mut self) {
        unsafe {
            binding::LZ4F_freeDecompressionContext(self.ctx.as_ptr());
        }
    }
}
