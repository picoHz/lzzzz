#![allow(unsafe_code)]

use super::Dictionary;
use crate::{
    binding,
    binding::{LZ4FCompressionCtx, LZ4FCompressionDict, LZ4FCompressionOptions},
    common,
    lz4f::Preferences,
    Error, Result,
};

use libc::{c_void, size_t};
use std::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

pub const HEADER_SIZE_MAX: usize = 19;

pub struct CompressionContext {
    ctx: NonNull<LZ4FCompressionCtx>,
    dict: Option<Dictionary>,
}

impl CompressionContext {
    pub fn new(dict: Option<Dictionary>) -> Result<Self> {
        let mut ctx = MaybeUninit::<*mut LZ4FCompressionCtx>::uninit();
        unsafe {
            let code = binding::LZ4F_createCompressionContext(
                ctx.as_ptr() as *mut *mut binding::LZ4FCompressionCtx,
                binding::LZ4F_getVersion(),
            );
            common::result_from_code(code).and_then(|_| {
                NonNull::new(ctx.assume_init())
                    .ok_or(Error::Generic)
                    .map(|ctx| Self { ctx, dict })
            })
        }
    }

    pub fn begin(&mut self, dst: &mut [u8], prefs: &Preferences) -> Result<usize> {
        let code = unsafe {
            if let Some(dict) = &self.dict {
                binding::LZ4F_compressBegin_usingCDict(
                    self.ctx.as_ptr(),
                    dst.as_mut_ptr() as *mut c_void,
                    dst.len() as size_t,
                    (*dict.0).0.as_ptr(),
                    prefs as *const Preferences,
                )
            } else {
                binding::LZ4F_compressBegin(
                    self.ctx.as_ptr(),
                    dst.as_mut_ptr() as *mut c_void,
                    dst.len() as size_t,
                    prefs,
                )
            }
        } as usize;
        common::result_from_code(code).map(|_| code)
    }

    pub fn update(&mut self, dst: &mut [u8], src: &[u8], stable_src: bool) -> Result<usize> {
        let opt = LZ4FCompressionOptions::stable(stable_src);
        let code = unsafe {
            binding::LZ4F_compressUpdate(
                self.ctx.as_ptr(),
                dst.as_mut_ptr() as *mut c_void,
                dst.len() as size_t,
                src.as_ptr() as *const c_void,
                src.len() as size_t,
                &opt as *const LZ4FCompressionOptions,
            )
        } as usize;
        common::result_from_code(code).map(|_| code)
    }

    pub fn flush(&mut self, dst: &mut [u8], stable_src: bool) -> Result<usize> {
        let opt = LZ4FCompressionOptions::stable(stable_src);
        let code = unsafe {
            binding::LZ4F_flush(
                self.ctx.as_ptr(),
                dst.as_mut_ptr() as *mut c_void,
                dst.len() as size_t,
                &opt as *const LZ4FCompressionOptions,
            )
        } as usize;
        common::result_from_code(code).map(|_| code)
    }

    pub fn end(&mut self, dst: &mut [u8], stable_src: bool) -> Result<usize> {
        let opt = LZ4FCompressionOptions::stable(stable_src);
        let code = unsafe {
            binding::LZ4F_compressEnd(
                self.ctx.as_ptr(),
                dst.as_mut_ptr() as *mut c_void,
                dst.len() as size_t,
                &opt as *const LZ4FCompressionOptions,
            )
        } as usize;
        common::result_from_code(code).map(|_| code)
    }

    pub fn compress_bound(src_size: usize, prefs: &Preferences) -> usize {
        unsafe { binding::LZ4F_compressBound(src_size as size_t, prefs as *const Preferences) }
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
pub struct LZ4Buffer {
    data: Vec<u8>,
    prev_size: usize,
}

impl LZ4Buffer {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn grow(&mut self, size: usize, prefs: &Preferences) {
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

pub struct DictionaryHandle(NonNull<LZ4FCompressionDict>);

unsafe impl Send for DictionaryHandle {}
unsafe impl Sync for DictionaryHandle {}

impl DictionaryHandle {
    pub fn new(data: &[u8]) -> Result<Self> {
        let dict = unsafe {
            binding::LZ4F_createCDict(data.as_ptr() as *const c_void, data.len() as size_t)
        };
        NonNull::new(dict)
            .ok_or(Error::Generic)
            .map(|ctx| Self(ctx))
    }
}

impl Drop for DictionaryHandle {
    fn drop(&mut self) {
        unsafe {
            binding::LZ4F_freeCDict(self.0.as_ptr());
        }
    }
}
