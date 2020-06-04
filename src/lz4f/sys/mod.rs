#![allow(unsafe_code)]

mod binding;

use crate::Result;
use libc::{c_void, size_t};
use std::ffi::CStr;

pub struct CompressionContext {
    ctx: *mut binding::CompressionContext,
}

impl CompressionContext {
    pub fn new() -> Result<Self> {
        let mut ctx: *mut binding::CompressionContext = std::ptr::null_mut();
        let code = unsafe {
            binding::LZ4F_createCompressionContext(
                &mut ctx as *mut *mut binding::CompressionContext,
                binding::LZ4F_getVersion(),
            )
        };
        Self::make_result(Self { ctx }, code)
    }

    pub fn begin(&mut self, dst: &mut [u8], prefs: Option<&binding::Preferences>) -> Result<usize> {
        let code = unsafe {
            binding::LZ4F_compressBegin(
                self.ctx,
                dst.as_mut_ptr() as *mut c_void,
                dst.len() as size_t,
                prefs
                    .map(|p| p as *const binding::Preferences)
                    .unwrap_or(std::ptr::null()),
            )
        };
        Self::make_result(code as usize, code)
    }

    pub fn update(
        &mut self,
        dst: &mut [u8],
        src: &[u8],
        opt: Option<&binding::CompressionOptions>,
    ) -> Result<usize> {
        let code = unsafe {
            binding::LZ4F_compressUpdate(
                self.ctx,
                dst.as_mut_ptr() as *mut c_void,
                dst.len() as size_t,
                src.as_ptr() as *const c_void,
                src.len() as size_t,
                opt.map(|p| p as *const binding::CompressionOptions)
                    .unwrap_or(std::ptr::null()),
            )
        };
        Self::make_result(code as usize, code)
    }

    pub fn flush(
        &mut self,
        dst: &mut [u8],
        opt: Option<&binding::CompressionOptions>,
    ) -> Result<usize> {
        let code = unsafe {
            binding::LZ4F_flush(
                self.ctx,
                dst.as_mut_ptr() as *mut c_void,
                dst.len() as size_t,
                opt.map(|p| p as *const binding::CompressionOptions)
                    .unwrap_or(std::ptr::null()),
            )
        };
        Self::make_result(code as usize, code)
    }

    pub fn end(
        &mut self,
        dst: &mut [u8],
        opt: Option<&binding::CompressionOptions>,
    ) -> Result<usize> {
        let code = unsafe {
            binding::LZ4F_compressEnd(
                self.ctx,
                dst.as_mut_ptr() as *mut c_void,
                dst.len() as size_t,
                opt.map(|p| p as *const binding::CompressionOptions)
                    .unwrap_or(std::ptr::null()),
            )
        };
        Self::make_result(code as usize, code)
    }

    pub fn compress_bound(src_size: usize, prefs: Option<&binding::Preferences>) -> usize {
        unsafe {
            binding::LZ4F_compressBound(
                src_size as size_t,
                prefs
                    .map(|p| p as *const binding::Preferences)
                    .unwrap_or(std::ptr::null()),
            )
        }
    }

    fn make_result<T>(data: T, code: size_t) -> Result<T> {
        unsafe {
            if binding::LZ4F_isError(code) != 0 {
                Err(CStr::from_ptr(binding::LZ4F_getErrorName(code))
                    .to_str()
                    .map_err(|_| "Invalid UTF-8")?)
            } else {
                Ok(data)
            }
        }
    }
}

impl Drop for CompressionContext {
    fn drop(&mut self) {
        debug_assert!(!self.ctx.is_null());
        unsafe {
            binding::LZ4F_freeCompressionContext(self.ctx);
        }
    }
}
