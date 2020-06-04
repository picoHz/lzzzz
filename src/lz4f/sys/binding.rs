use crate::lz4f::{BlockChecksum, BlockMode, BlockSize, ContentChecksum, FrameType};
use libc::{c_char, c_int, c_uint, c_ulonglong, c_void, size_t};

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

#[repr(C)]
pub struct CompressionContext {
    _private: [u8; 0],
}

#[link(name = "lz4")]
extern "C" {
    pub fn LZ4F_getVersion() -> c_uint;
    pub fn LZ4F_createCompressionContext(
        ctx: *mut *mut CompressionContext,
        version: c_uint,
    ) -> size_t;
    pub fn LZ4F_freeCompressionContext(ctx: *mut CompressionContext);
    pub fn LZ4F_compressBegin(
        ctx: *mut CompressionContext,
        dst_buffer: *mut c_void,
        dst_capacity: size_t,
        prefs: *const Preferences,
    ) -> size_t;
    pub fn LZ4F_compressBound(src_size: size_t, prefs: *const Preferences) -> size_t;
    pub fn LZ4F_compressUpdate(
        ctx: *mut CompressionContext,
        dst_buffer: *mut c_void,
        dst_capacity: size_t,
        src_buffer: *const c_void,
        src_size: size_t,
        opt: *const CompressionOptions,
    ) -> size_t;
    pub fn LZ4F_flush(
        ctx: *mut CompressionContext,
        dst_buffer: *mut c_void,
        dst_capacity: size_t,
        opt: *const CompressionOptions,
    ) -> size_t;
    pub fn LZ4F_compressEnd(
        ctx: *mut CompressionContext,
        dst_buffer: *mut c_void,
        dst_capacity: size_t,
        opt: *const CompressionOptions,
    ) -> size_t;
    pub fn LZ4F_isError(code: size_t) -> c_uint;
    pub fn LZ4F_getErrorName(code: size_t) -> *const c_char;
}

#[cfg(test)]
mod tests {
    #[test]
    fn call_c_api() {
        use super::{
            CompressionContext, LZ4F_createCompressionContext, LZ4F_freeCompressionContext,
            LZ4F_getVersion,
        };
        unsafe {
            let mut ctx: *mut CompressionContext = std::ptr::null_mut();
            assert_eq!(
                LZ4F_createCompressionContext(
                    &mut ctx as *mut *mut CompressionContext,
                    LZ4F_getVersion()
                ),
                0
            );
            assert!(!ctx.is_null());
            LZ4F_freeCompressionContext(ctx);
        }
    }
}
