use crate::lz4f::{FrameInfo, Preferences};
use libc::{c_char, c_int, c_uint, c_void, size_t};

#[cfg(feature = "lz4-use-stack")]
mod lz4 {
    const LZ4_MEMORY_USAGE: usize = 14;
    const LZ4_STREAMSIZE_U64: usize = (1 << (LZ4_MEMORY_USAGE - 3)) + 4;

    #[repr(C)]
    pub struct LZ4Stream {
        _private: [u64; LZ4_STREAMSIZE_U64],
        _pin: std::marker::PhantomPinned,
    }
}

#[cfg(not(feature = "lz4-use-stack"))]
mod lz4 {
    #[repr(C)]
    pub struct LZ4Stream {
        _private: [u8; 0],
    }
}

#[cfg(feature = "lz4hc-use-stack")]
mod lz4_hc {
    const LZ4HC_HASH_LOG: usize = 15;
    const LZ4HC_HASHTABLESIZE: usize = 1 << LZ4HC_HASH_LOG;
    const LZ4HC_DICTIONARY_LOGSIZE: usize = 16;
    const LZ4HC_MAXD: usize = 1 << LZ4HC_DICTIONARY_LOGSIZE;
    const LZ4_STREAMHCSIZE: usize = 4 * LZ4HC_HASHTABLESIZE + 2 * LZ4HC_MAXD + 56;
    const LZ4_STREAMHCSIZE_SIZET: usize = LZ4_STREAMHCSIZE / std::mem::size_of::<size_t>();

    #[repr(C)]
    pub struct LZ4StreamHC {
        _private: [size_t; LZ4_STREAMHCSIZE_SIZET],
        _pin: std::marker::PhantomPinned,
    }
}

#[cfg(not(feature = "lz4hc-use-stack"))]
mod lz4_hc {
    #[repr(C)]
    pub struct LZ4StreamHC {
        _private: [u8; 0],
    }
}

pub use lz4::LZ4Stream;
pub use lz4_hc::LZ4StreamHC;

#[repr(C)]
pub struct LZ4DecStream {
    _private: [u8; 0],
}

#[repr(C)]
pub struct LZ4FCompressionCtx {
    _private: [u8; 0],
}

#[repr(C)]
pub struct LZ4FDecompressionCtx {
    _private: [u8; 0],
}

#[repr(C)]
pub struct LZ4FCompressionDict {
    _private: [u8; 0],
}

#[derive(Debug, Default, Copy, Clone)]
#[repr(C)]
pub struct LZ4FCompressionOptions {
    pub stable_src: c_uint,
    pub _reserved: [c_uint; 3],
}

impl LZ4FCompressionOptions {
    pub fn stable(stable: bool) -> Self {
        Self {
            stable_src: if stable { 1 } else { 0 },
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
#[repr(C)]
pub struct LZ4FDecompressionOptions {
    pub stable_dst: c_uint,
    pub _reserved: [c_uint; 3],
}

impl LZ4FDecompressionOptions {
    pub fn stable(stable: bool) -> Self {
        Self {
            stable_dst: if stable { 1 } else { 0 },
            ..Default::default()
        }
    }
}

#[link(name = "lz4")]
extern "C" {
    pub fn LZ4_versionNumber() -> c_int;
    pub fn LZ4_versionString() -> *const c_char;

    pub fn LZ4_compressBound(input_size: c_int) -> c_int;
    pub fn LZ4_sizeofState() -> c_int;
    pub fn LZ4_compress_fast_extState(
        state: *mut c_void,
        src: *const c_char,
        dst: *mut c_char,
        src_size: c_int,
        dst_capacity: c_int,
        acceleration: c_int,
    ) -> c_int;
    pub fn LZ4_compress_fast_extState_fastReset(
        state: *mut c_void,
        src: *const c_char,
        dst: *mut c_char,
        src_size: c_int,
        dst_capacity: c_int,
        acceleration: c_int,
    ) -> c_int;
    pub fn LZ4_decompress_safe(
        src: *const c_char,
        dst: *mut c_char,
        compressed_size: c_int,
        dst_capacity: c_int,
    ) -> c_int;
    pub fn LZ4_decompress_safe_partial(
        src: *const c_char,
        dst: *mut c_char,
        src_size: c_int,
        target_output_size: c_int,
        dst_capacity: c_int,
    ) -> c_int;
    pub fn LZ4_decompress_safe_usingDict(
        src: *const c_char,
        dst: *mut c_char,
        compressed_size: c_int,
        dst_capacity: c_int,
        dict_start: *const c_char,
        dict_size: c_int,
    ) -> c_int;

    pub fn LZ4_sizeofStateHC() -> c_int;
    pub fn LZ4_compress_HC_extStateHC(
        state: *mut c_void,
        src: *const c_char,
        dst: *mut c_char,
        src_size: c_int,
        dst_capacity: c_int,
        compression_level: c_int,
    ) -> c_int;
    pub fn LZ4_compress_HC_extStateHC_fastReset(
        state: *mut c_void,
        src: *const c_char,
        dst: *mut c_char,
        src_size: c_int,
        dst_capacity: c_int,
        compression_level: c_int,
    ) -> c_int;
    pub fn LZ4_compress_HC_destSize(
        state: *mut c_void,
        src: *const c_char,
        dst: *mut c_char,
        src_size_ptr: *mut c_int,
        target_dst_dize: c_int,
        compression_level: c_int,
    ) -> c_int;
    pub fn LZ4_initStreamHC(buffer: *mut c_void, size: size_t) -> *mut LZ4StreamHC;

    pub fn LZ4_createStreamHC() -> *mut LZ4StreamHC;
    pub fn LZ4_freeStreamHC(ptr: *mut LZ4StreamHC) -> c_int;
    pub fn LZ4_resetStreamHC_fast(ptr: *mut LZ4StreamHC, compression_level: c_int);

    pub fn LZ4F_getVersion() -> c_uint;
    pub fn LZ4F_compressBound(src_size: size_t, prefs: *const Preferences) -> size_t;
    pub fn LZ4F_compressFrame(
        dst_buffer: *mut c_void,
        dst_capacity: size_t,
        src_buffer: *const c_void,
        src_size: size_t,
        prefs: *const Preferences,
    ) -> size_t;
    pub fn LZ4F_decompress(
        ctx: *mut LZ4FDecompressionCtx,
        dst_buffer: *mut c_void,
        dst_size_ptr: *mut size_t,
        src_buffer: *const c_void,
        src_size_ptr: *mut size_t,
        opt: *const LZ4FDecompressionOptions,
    ) -> size_t;
    pub fn LZ4F_decompress_usingDict(
        ctx: *mut LZ4FDecompressionCtx,
        dst_buffer: *mut c_void,
        dst_size_ptr: *mut size_t,
        src_buffer: *const c_void,
        src_size_ptr: *mut size_t,
        dict: *const c_void,
        dict_size: size_t,
        opt: *const LZ4FDecompressionOptions,
    ) -> size_t;
    pub fn LZ4_createStream() -> *mut LZ4Stream;
    pub fn LZ4_freeStream(ptr: *mut LZ4Stream) -> c_int;
    pub fn LZ4_initStream(buffer: *mut c_void, size: size_t) -> *mut LZ4Stream;
    pub fn LZ4_createStreamDecode() -> *mut LZ4DecStream;
    pub fn LZ4_freeStreamDecode(stream: *mut LZ4DecStream) -> c_int;

    pub fn LZ4F_createCDict(
        dict_buffer: *const c_void,
        dict_size: size_t,
    ) -> *mut LZ4FCompressionDict;
    pub fn LZ4F_freeCDict(dict: *mut LZ4FCompressionDict);

    pub fn LZ4F_isError(code: size_t) -> c_uint;
    pub fn LZ4F_getErrorName(code: size_t) -> *const c_char;

    pub fn LZ4F_createCompressionContext(
        ctx: *mut *mut LZ4FCompressionCtx,
        version: c_uint,
    ) -> size_t;
    pub fn LZ4F_freeCompressionContext(ctx: *mut LZ4FCompressionCtx);
    pub fn LZ4F_compressBegin(
        ctx: *mut LZ4FCompressionCtx,
        dst_buffer: *mut c_void,
        dst_capacity: size_t,
        prefs: *const Preferences,
    ) -> size_t;
    pub fn LZ4F_compressBegin_usingCDict(
        ctx: *mut LZ4FCompressionCtx,
        dst_buffer: *mut c_void,
        dst_capacity: size_t,
        dist: *const LZ4FCompressionDict,
        prefs: *const Preferences,
    ) -> size_t;
    pub fn LZ4F_compressUpdate(
        ctx: *mut LZ4FCompressionCtx,
        dst_buffer: *mut c_void,
        dst_capacity: size_t,
        src_buffer: *const c_void,
        src_size: size_t,
        opt: *const LZ4FCompressionOptions,
    ) -> size_t;
    pub fn LZ4F_flush(
        ctx: *mut LZ4FCompressionCtx,
        dst_buffer: *mut c_void,
        dst_capacity: size_t,
        opt: *const LZ4FCompressionOptions,
    ) -> size_t;
    pub fn LZ4F_compressEnd(
        ctx: *mut LZ4FCompressionCtx,
        dst_buffer: *mut c_void,
        dst_capacity: size_t,
        opt: *const LZ4FCompressionOptions,
    ) -> size_t;
    pub fn LZ4F_createDecompressionContext(
        ctx: *mut *mut LZ4FDecompressionCtx,
        version: c_uint,
    ) -> size_t;
    pub fn LZ4F_freeDecompressionContext(ctx: *mut LZ4FDecompressionCtx) -> size_t;
    pub fn LZ4F_resetDecompressionContext(ctx: *mut LZ4FDecompressionCtx);
    pub fn LZ4F_getFrameInfo(
        ctx: *mut LZ4FDecompressionCtx,
        frame_info_ptr: *mut FrameInfo,
        src_buffer: *const c_void,
        src_size_ptr: *mut size_t,
    ) -> size_t;
}
