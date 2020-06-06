#![allow(unsafe_code)]

use super::api::{CompressionOptions, Preferences};
use libc::{c_char, c_uint, c_void, size_t};

#[repr(C)]
pub struct CompressionCtx {
    _private: [u8; 0],
}

#[repr(C)]
pub struct CompressionDict {
    _private: [u8; 0],
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
        prefs: *const Preferences,
    ) -> size_t;
    pub fn LZ4F_compressBound(src_size: size_t, prefs: *const Preferences) -> size_t;
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
    pub fn LZ4F_createCDict(dict_buffer: *const c_void, dict_size: size_t) -> *mut CompressionDict;
    pub fn LZ4F_freeCDict(dict: *mut CompressionDict);
    pub fn LZ4F_isError(code: size_t) -> c_uint;
    pub fn LZ4F_getErrorName(code: size_t) -> *const c_char;
}

#[cfg(test)]
mod tests {
    #[test]
    fn call_c_api() {
        use super::{
            CompressionCtx, LZ4F_createCompressionContext, LZ4F_freeCompressionContext,
            LZ4F_getVersion,
        };
        unsafe {
            let mut ctx: *mut CompressionCtx = std::ptr::null_mut();
            assert_eq!(
                LZ4F_createCompressionContext(
                    &mut ctx as *mut *mut CompressionCtx,
                    LZ4F_getVersion()
                ),
                0
            );
            assert!(!ctx.is_null());
            LZ4F_freeCompressionContext(ctx);
        }
    }
}
