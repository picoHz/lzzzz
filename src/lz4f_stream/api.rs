#![allow(unsafe_code)]

use super::{
    binding,
    binding::{
        CompressionCtx, CompressionDict, CompressionOptions, DecompressionCtx, DecompressionOptions,
    },
    Dictionary,
};
use crate::{lz4f::api::Pref, LZ4Error, Result};

use libc::{c_int, c_uint, c_ulonglong, c_void, size_t};
use std::{
    ffi::CStr,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

pub const HEADER_SIZE_MIN: usize = 7;
pub const HEADER_SIZE_MAX: usize = 19;

pub struct CompressionContext {
    ctx: NonNull<CompressionCtx>,
    dict: Option<Dictionary>,
}

impl CompressionContext {
    pub fn new(dict: Option<Dictionary>) -> Result<Self> {
        let mut ctx: *mut CompressionCtx = std::ptr::null_mut();
        let code = unsafe {
            binding::LZ4F_createCompressionContext(
                &mut ctx as *mut *mut CompressionCtx,
                binding::LZ4F_getVersion(),
            )
        };
        make_result(
            Self {
                ctx: NonNull::new(ctx).unwrap(),
                dict,
            },
            code,
        )
    }

    pub fn begin(&mut self, dst: &mut [u8], prefs: &Pref) -> Result<usize> {
        let code = unsafe {
            if let Some(dict) = &self.dict {
                binding::LZ4F_compressBegin_usingCDict(
                    self.ctx.as_ptr(),
                    dst.as_mut_ptr() as *mut c_void,
                    dst.len() as size_t,
                    (*dict.0).0.as_ptr(),
                    prefs as *const Pref,
                )
            } else {
                binding::LZ4F_compressBegin(
                    self.ctx.as_ptr(),
                    dst.as_mut_ptr() as *mut c_void,
                    dst.len() as size_t,
                    prefs,
                )
            }
        };
        make_result(code as usize, code)
    }

    pub fn update(
        &mut self,
        dst: &mut [u8],
        src: &[u8],
        opt: Option<&CompressionOptions>,
    ) -> Result<usize> {
        let code = unsafe {
            binding::LZ4F_compressUpdate(
                self.ctx.as_ptr(),
                dst.as_mut_ptr() as *mut c_void,
                dst.len() as size_t,
                src.as_ptr() as *const c_void,
                src.len() as size_t,
                opt.map(|p| p as *const CompressionOptions)
                    .unwrap_or(std::ptr::null()),
            )
        };
        make_result(code as usize, code)
    }

    pub fn flush(&mut self, dst: &mut [u8], opt: Option<&CompressionOptions>) -> Result<usize> {
        let code = unsafe {
            binding::LZ4F_flush(
                self.ctx.as_ptr(),
                dst.as_mut_ptr() as *mut c_void,
                dst.len() as size_t,
                opt.map(|p| p as *const CompressionOptions)
                    .unwrap_or(std::ptr::null()),
            )
        };
        make_result(code as usize, code)
    }

    pub fn end(&mut self, dst: &mut [u8], opt: Option<&CompressionOptions>) -> Result<usize> {
        let code = unsafe {
            binding::LZ4F_compressEnd(
                self.ctx.as_ptr(),
                dst.as_mut_ptr() as *mut c_void,
                dst.len() as size_t,
                opt.map(|p| p as *const CompressionOptions)
                    .unwrap_or(std::ptr::null()),
            )
        };
        make_result(code as usize, code)
    }

    pub fn compress_bound(src_size: usize, prefs: &Pref) -> usize {
        unsafe {
            crate::lz4f::binding::LZ4F_compressBound(src_size as size_t, prefs as *const Pref)
        }
    }
}

fn make_result<T>(data: T, code: size_t) -> Result<T> {
    unsafe {
        if crate::lz4f::binding::LZ4F_isError(code) != 0 {
            Err(LZ4Error::from(
                CStr::from_ptr(crate::lz4f::binding::LZ4F_getErrorName(code))
                    .to_str()
                    .map_err(|_| LZ4Error::from("Invalid UTF-8"))?,
            ))
        } else {
            Ok(data)
        }
    }
}

impl Drop for CompressionContext {
    fn drop(&mut self) {
        unsafe {
            binding::LZ4F_freeCompressionContext(self.ctx.as_ptr());
        }
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct LZ4Buffer {
    data: Vec<u8>,
    prev_size: usize,
}

impl LZ4Buffer {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn grow(&mut self, size: usize, prefs: &Pref) {
        if self.prev_size == 0 || size + 1 > self.prev_size {
            let len = CompressionContext::compress_bound(size, prefs) + HEADER_SIZE_MAX;
            if len > self.data.len() {
                self.data.reserve(len - self.data.len());

                #[allow(unsafe_code)]
                unsafe {
                    self.data.set_len(len)
                };
            }
            self.prev_size = size + 1;
        }
    }
}

impl Deref for LZ4Buffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for LZ4Buffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

pub(crate) struct DictionaryHandle(NonNull<CompressionDict>);

unsafe impl Send for DictionaryHandle {}
unsafe impl Sync for DictionaryHandle {}

impl DictionaryHandle {
    pub fn new(data: &[u8]) -> Self {
        let dict = unsafe {
            binding::LZ4F_createCDict(data.as_ptr() as *const c_void, data.len() as size_t)
        };
        Self(NonNull::new(dict).unwrap())
    }
}

impl Drop for DictionaryHandle {
    fn drop(&mut self) {
        unsafe {
            binding::LZ4F_freeCDict(self.0.as_ptr());
        }
    }
}
