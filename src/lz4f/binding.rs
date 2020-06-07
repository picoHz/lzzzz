#![allow(unsafe_code)]

use super::{api::Pref, FrameInfo};
use libc::{c_char, c_uint, c_void, size_t};

#[repr(C)]
pub struct CompressionCtx {
    _private: [u8; 0],
}

#[repr(C)]
pub struct DecompressionCtx {
    _private: [u8; 0],
}

#[repr(C)]
pub struct CompressionDict {
    _private: [u8; 0],
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct DecompressionOptions {
    pub stable_dst: c_uint,
    pub _reserved: [c_uint; 3],
}

#[link(name = "lz4")]
extern "C" {
    pub fn LZ4F_compressBound(src_size: size_t, prefs: *const Pref) -> size_t;
    pub fn LZ4F_decompress(
        ctx: *mut DecompressionCtx,
        dst_buffer: *mut c_void,
        dst_size_ptr: *mut size_t,
        src_buffer: *const c_void,
        src_size_ptr: *mut size_t,
        opt: *const DecompressionOptions,
    ) -> size_t;
    pub fn LZ4F_decompress_usingDict(
        ctx: *mut DecompressionCtx,
        dst_buffer: *mut c_void,
        dst_size_ptr: *mut size_t,
        src_buffer: *const c_void,
        src_size_ptr: *mut size_t,
        dict: *const c_void,
        dict_size: size_t,
        opt: *const DecompressionOptions,
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
