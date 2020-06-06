//! LZ4F Low-level API
//!
//! This module provides a thin memory-safe wrapper of liblz4.

#![allow(unsafe_code)]

use super::{
    binding, AutoFlush, BlockChecksum, BlockMode, BlockSize, ContentChecksum, Dictionary,
    FavorDecSpeed,
};
use crate::{LZ4Error, Result};

use binding::{CompressionCtx, CompressionDict};
use libc::{c_int, c_uint, c_ulonglong, c_void, size_t};
use std::{ffi::CStr, ptr::NonNull};

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

pub struct CompressionContext {
    ctx: NonNull<CompressionCtx>,
    dict: Option<Dictionary>,
}

impl CompressionContext {
    pub fn new() -> Result<Self> {
        let mut ctx: *mut CompressionCtx = std::ptr::null_mut();
        let code = unsafe {
            binding::LZ4F_createCompressionContext(
                &mut ctx as *mut *mut CompressionCtx,
                binding::LZ4F_getVersion(),
            )
        };
        Self::make_result(
            Self {
                ctx: NonNull::new(ctx).unwrap(),
                dict: None,
            },
            code,
        )
    }

    pub fn begin(
        &mut self,
        dst: &mut [u8],
        prefs: Option<&Preferences>,
        dict: Option<Dictionary>,
    ) -> Result<usize> {
        let prefs = prefs
            .map(|p| p as *const Preferences)
            .unwrap_or(std::ptr::null());
        let code = unsafe {
            if let Some(dict) = &dict {
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
        self.dict = dict;
        Self::make_result(code as usize, code)
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
        Self::make_result(code as usize, code)
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
        Self::make_result(code as usize, code)
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
        Self::make_result(code as usize, code)
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
}

impl Drop for CompressionContext {
    fn drop(&mut self) {
        unsafe {
            binding::LZ4F_freeCompressionContext(self.ctx.as_ptr());
        }
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
