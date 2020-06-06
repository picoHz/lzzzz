#![allow(unsafe_code)]

use super::{
    binding, AutoFlush, BlockChecksum, BlockMode, BlockSize, ContentChecksum, Dictionary,
    FavorDecSpeed,
};
use crate::{LZ4Error, Result};

use binding::{CompressionCtx, CompressionDict, DecompressionCtx};
use libc::{c_int, c_uint, c_ulonglong, c_void, size_t};
use std::{
    ffi::CStr,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

pub const HEADER_SIZE_MIN: usize = 7;
pub const HEADER_SIZE_MAX: usize = 19;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum FrameType {
    Frame = 0,
    SkippableFrame = 1,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct FrameInfo {
    pub block_size: BlockSize,
    pub block_mode: BlockMode,
    pub content_checksum: ContentChecksum,
    pub frame_type: FrameType,
    pub content_size: c_ulonglong,
    pub dict_id: c_uint,
    pub block_checksum: BlockChecksum,
}

impl Default for FrameInfo {
    fn default() -> Self {
        Self {
            block_size: BlockSize::Default,
            block_mode: BlockMode::Linked,
            content_checksum: ContentChecksum::Disabled,
            frame_type: FrameType::Frame,
            content_size: 0,
            dict_id: 0,
            block_checksum: BlockChecksum::Disabled,
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Preferences {
    pub frame_info: FrameInfo,
    pub compression_level: c_int,
    pub auto_flush: AutoFlush,
    pub favor_dec_speed: FavorDecSpeed,
    pub _reserved: [c_uint; 3],
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            frame_info: FrameInfo::default(),
            compression_level: 0,
            auto_flush: AutoFlush::Disabled,
            favor_dec_speed: FavorDecSpeed::Disabled,
            _reserved: [0; 3],
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct CompressionOptions {
    pub stable_src: c_uint,
    pub _reserved: [c_uint; 3],
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct DecompressionOptions {
    pub stable_dst: c_uint,
    pub _reserved: [c_uint; 3],
}

pub struct DecompressionContext {
    ctx: NonNull<DecompressionCtx>,
}

impl DecompressionContext {
    pub fn new() -> Result<Self> {
        let mut ctx: *mut DecompressionCtx = std::ptr::null_mut();
        let code = unsafe {
            binding::LZ4F_createDecompressionContext(
                &mut ctx as *mut *mut DecompressionCtx,
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
        dst: &mut [u8],
        src: &[u8],
        opt: Option<&DecompressionOptions>,
    ) -> Result<usize> {
        let mut dst_size = dst.len();
        let mut src_size = src.len();
        let code = unsafe {
            binding::LZ4F_decompress(
                self.ctx.as_ptr(),
                dst.as_mut_ptr() as *mut c_void,
                &mut dst_size as *mut size_t,
                src.as_ptr() as *const c_void,
                &mut src_size as *mut size_t,
                opt.map(|p| p as *const DecompressionOptions)
                    .unwrap_or(std::ptr::null()),
            )
        };
        make_result(code as usize, code)
    }
}

impl Drop for DecompressionContext {
    fn drop(&mut self) {
        unsafe {
            binding::LZ4F_freeDecompressionContext(self.ctx.as_ptr());
        }
    }
}

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

    pub fn begin(&mut self, dst: &mut [u8], prefs: Option<&Preferences>) -> Result<usize> {
        let prefs = prefs
            .map(|p| p as *const Preferences)
            .unwrap_or(std::ptr::null());
        let code = unsafe {
            if let Some(dict) = &self.dict {
                binding::LZ4F_compressBegin_usingCDict(
                    self.ctx.as_ptr(),
                    dst.as_mut_ptr() as *mut c_void,
                    dst.len() as size_t,
                    (*dict.0).0.as_ptr(),
                    prefs,
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

    pub fn compress_bound(src_size: usize, prefs: Option<&Preferences>) -> usize {
        unsafe {
            binding::LZ4F_compressBound(
                src_size as size_t,
                prefs
                    .map(|p| p as *const Preferences)
                    .unwrap_or(std::ptr::null()),
            )
        }
    }
}

fn make_result<T>(data: T, code: size_t) -> Result<T> {
    unsafe {
        if binding::LZ4F_isError(code) != 0 {
            Err(LZ4Error::from(
                CStr::from_ptr(binding::LZ4F_getErrorName(code))
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

    pub fn grow(&mut self, size: usize, prefs: Option<&Preferences>) {
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
