#![allow(unsafe_code)]

use super::Dictionary;
use crate::{
    binding,
    binding::{
        LZ4FCompressionCtx, LZ4FCompressionDict, LZ4FCompressionOptions, LZ4FDecompressionCtx,
        LZ4FDecompressionOptions,
    },
    common,
    lz4f::{FrameInfo, Preferences},
    Error, Report, Result,
};

use std::{mem::MaybeUninit, os::raw::c_void, ptr::NonNull};

pub const LZ4F_HEADER_SIZE_MAX: usize = 19;

pub struct CompressionContext {
    ctx: NonNull<LZ4FCompressionCtx>,
    dict: Option<Dictionary>,
}

impl CompressionContext {
    pub fn new(dict: Option<Dictionary>) -> Result<Self> {
        let ctx = MaybeUninit::<*mut LZ4FCompressionCtx>::uninit();
        unsafe {
            let code = binding::LZ4F_createCompressionContext(
                ctx.as_ptr() as *mut *mut binding::LZ4FCompressionCtx,
                binding::LZ4F_getVersion(),
            );
            common::result_from_code(code).and_then(|_| {
                Ok(Self {
                    ctx: NonNull::new(ctx.assume_init()).ok_or(Error::NullPointerUnexpected)?,
                    dict,
                })
            })
        }
    }

    pub fn begin(&mut self, dst: &mut [u8], prefs: &Preferences) -> Result<usize> {
        let code = unsafe {
            if let Some(dict) = &self.dict {
                binding::LZ4F_compressBegin_usingCDict(
                    self.ctx.as_ptr(),
                    dst.as_mut_ptr() as *mut c_void,
                    dst.len(),
                    (*dict.handle()).0.as_ptr(),
                    prefs as *const Preferences,
                )
            } else {
                binding::LZ4F_compressBegin(
                    self.ctx.as_ptr(),
                    dst.as_mut_ptr() as *mut c_void,
                    dst.len(),
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
                dst.len(),
                src.as_ptr() as *const c_void,
                src.len(),
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
                dst.len(),
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
                dst.len(),
                &opt as *const LZ4FCompressionOptions,
            )
        } as usize;
        common::result_from_code(code).map(|_| code)
    }

    pub fn compress_bound(src_size: usize, prefs: &Preferences) -> usize {
        unsafe { binding::LZ4F_compressBound(src_size as usize, prefs as *const Preferences) }
    }
}

impl Drop for CompressionContext {
    fn drop(&mut self) {
        unsafe {
            binding::LZ4F_freeCompressionContext(self.ctx.as_ptr());
        }
    }
}

pub struct DecompressionContext {
    ctx: NonNull<LZ4FDecompressionCtx>,
}

impl DecompressionContext {
    pub fn new() -> Result<Self> {
        let ctx = MaybeUninit::<*mut LZ4FDecompressionCtx>::uninit();
        unsafe {
            let code = binding::LZ4F_createDecompressionContext(
                ctx.as_ptr() as *mut *mut binding::LZ4FDecompressionCtx,
                binding::LZ4F_getVersion(),
            );
            common::result_from_code(code).and_then(|_| {
                Ok(Self {
                    ctx: NonNull::new(ctx.assume_init()).ok_or(Error::NullPointerUnexpected)?,
                })
            })
        }
    }

    pub fn get_frame_info(&self, src: &[u8]) -> Result<(FrameInfo, usize)> {
        let mut info = MaybeUninit::<FrameInfo>::uninit();
        let mut src_len = src.len();
        let code = unsafe {
            binding::LZ4F_getFrameInfo(
                self.ctx.as_ptr(),
                info.as_mut_ptr() as *mut FrameInfo,
                src.as_ptr() as *const c_void,
                &mut src_len as *mut usize,
            )
        };
        common::result_from_code(code).map(|_| (unsafe { info.assume_init() }, src_len as usize))
    }

    pub fn decompress_dict(
        &mut self,
        src: &[u8],
        dst: &mut [u8],
        dict: &[u8],
        stable_dst: bool,
    ) -> Result<Report> {
        let mut dst_len = dst.len();
        let mut src_len = src.len();
        let opt = LZ4FDecompressionOptions::stable(stable_dst);
        let code = unsafe {
            binding::LZ4F_decompress_usingDict(
                self.ctx.as_ptr(),
                dst.as_mut_ptr() as *mut c_void,
                &mut dst_len as *mut usize,
                src.as_ptr() as *const c_void,
                &mut src_len as *mut usize,
                dict.as_ptr() as *const c_void,
                dict.len(),
                &opt as *const LZ4FDecompressionOptions,
            )
        };
        common::result_from_code(code).map(|_| Report {
            src_len: Some(src_len as usize),
            dst_len: dst_len as usize,
            expected_src_len: Some(code as usize),
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

pub fn compress_bound(input_size: usize, prefs: &Preferences) -> usize {
    unsafe {
        binding::LZ4F_compressBound(input_size as usize, prefs as *const Preferences) as usize
    }
}

pub fn compress(src: &[u8], dst: &mut [u8], prefs: &Preferences) -> Result<Report> {
    let code = unsafe {
        binding::LZ4F_compressFrame(
            dst.as_mut_ptr() as *mut c_void,
            dst.len(),
            src.as_ptr() as *const c_void,
            src.len(),
            prefs as *const Preferences,
        ) as usize
    };
    common::result_from_code(code).map(|_| Report {
        dst_len: code,
        ..Default::default()
    })
}

pub struct DictionaryHandle(NonNull<LZ4FCompressionDict>);

unsafe impl Send for DictionaryHandle {}
unsafe impl Sync for DictionaryHandle {}

impl DictionaryHandle {
    pub fn new(data: &[u8]) -> Result<Self> {
        let dict = unsafe { binding::LZ4F_createCDict(data.as_ptr() as *const c_void, data.len()) };
        NonNull::new(dict)
            .ok_or(Error::NullPointerUnexpected)
            .map(Self)
    }
}

impl Drop for DictionaryHandle {
    fn drop(&mut self) {
        unsafe {
            binding::LZ4F_freeCDict(self.0.as_ptr());
        }
    }
}
