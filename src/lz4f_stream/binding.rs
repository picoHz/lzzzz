#![allow(unsafe_code)]

use crate::lz4f::{api::Pref, binding::CompressionDict, FrameInfo};
use libc::{c_int, c_uint, c_ulonglong, c_void, size_t};

#[repr(C)]
pub struct CompressionCtx {
    _private: [u8; 0],
}

#[repr(C)]
pub struct DecompressionCtx {
    _private: [u8; 0],
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct CompressionOptions {
    pub stable_src: c_uint,
    pub _reserved: [c_uint; 3],
}

#[link(name = "lz4")]
extern "C" {
    pub fn LZ4F_getVersion() -> c_uint;

    pub fn LZ4F_createCompressionContext(ctx: *mut *mut CompressionCtx, version: c_uint) -> size_t;
    pub fn LZ4F_freeCompressionContext(ctx: *mut CompressionCtx);
    pub fn LZ4F_compressBegin(
        ctx: *mut CompressionCtx,
        dst_buffer: *mut c_void,
        dst_capacity: size_t,
        prefs: *const Pref,
    ) -> size_t;
    pub fn LZ4F_compressBegin_usingCDict(
        ctx: *mut CompressionCtx,
        dst_buffer: *mut c_void,
        dst_capacity: size_t,
        dist: *const CompressionDict,
        prefs: *const Pref,
    ) -> size_t;
    pub fn LZ4F_compressUpdate(
        ctx: *mut CompressionCtx,
        dst_buffer: *mut c_void,
        dst_capacity: size_t,
        src_buffer: *const c_void,
        src_size: size_t,
        opt: *const CompressionOptions,
    ) -> size_t;
    pub fn LZ4F_flush(
        ctx: *mut CompressionCtx,
        dst_buffer: *mut c_void,
        dst_capacity: size_t,
        opt: *const CompressionOptions,
    ) -> size_t;
    pub fn LZ4F_compressEnd(
        ctx: *mut CompressionCtx,
        dst_buffer: *mut c_void,
        dst_capacity: size_t,
        opt: *const CompressionOptions,
    ) -> size_t;

    pub fn LZ4F_createDecompressionContext(
        ctx: *mut *mut DecompressionCtx,
        version: c_uint,
    ) -> size_t;
    pub fn LZ4F_freeDecompressionContext(ctx: *mut DecompressionCtx) -> size_t;
    pub fn LZ4F_getFrameInfo(
        ctx: *mut DecompressionCtx,
        frame_info_ptr: *mut FrameInfo,
        src_buffer: *const c_void,
        src_size_ptr: *mut size_t,
    ) -> size_t;
}
