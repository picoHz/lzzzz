#![allow(unsafe_code)]

mod binding;
use crate::{Result, LZ4Error};

use libc::{c_int, c_uint, c_ulonglong, c_void, size_t};
use std::ffi::CStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum BlockSize {
    Default = 0,
    Max64KB = 4,
    Max256KB = 5,
    Max1MB = 6,
    Max4MB = 7,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum BlockMode {
    Linked = 0,
    Independent = 1,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum ContentChecksum {
    Disabled = 0,
    Enabled = 1,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum BlockChecksum {
    Disabled = 0,
    Enabled = 1,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum FrameType {
    Frame = 0,
    SkippableFrame = 1,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct FrameInfo {
    block_size: BlockSize,
    block_mode: BlockMode,
    content_checksum: ContentChecksum,
    frame_type: FrameType,
    content_size: c_ulonglong,
    dict_id: c_uint,
    block_ckecksum: BlockChecksum,
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
            block_ckecksum: BlockChecksum::Disabled,
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Preferences {
    frame_info: FrameInfo,
    compression_level: c_int,
    auto_flush: c_uint,
    favor_dec_speed: c_uint,
    _reserved: [c_uint; 3],
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            frame_info: FrameInfo::default(),
            compression_level: 0,
            auto_flush: 0,
            favor_dec_speed: 0,
            _reserved: [0; 3],
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct CompressionOptions {
    stable_src: c_uint,
    _reserved: [c_uint; 3],
}

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

    pub fn begin(&mut self, dst: &mut [u8], prefs: Option<&Preferences>) -> Result<usize> {
        let code = unsafe {
            binding::LZ4F_compressBegin(
                self.ctx,
                dst.as_mut_ptr() as *mut c_void,
                dst.len() as size_t,
                prefs
                    .map(|p| p as *const Preferences)
                    .unwrap_or(std::ptr::null()),
            )
        };
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
                self.ctx,
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
                self.ctx,
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
                self.ctx,
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
                Err(LZ4Error::new(CStr::from_ptr(binding::LZ4F_getErrorName(code))
                    .to_str()
                    .map_err(|_| LZ4Error::new("Invalid UTF-8"))?))
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
