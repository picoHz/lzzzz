use std::os::raw::{c_char, c_int, c_void};

#[repr(C)]
pub struct LZ4StreamHC {
    _private: [u8; 0],
}

extern "C" {
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

    pub fn LZ4_createStreamHC() -> *mut LZ4StreamHC;
    pub fn LZ4_freeStreamHC(ptr: *mut LZ4StreamHC) -> c_int;
    pub fn LZ4_resetStreamHC_fast(ptr: *mut LZ4StreamHC, compression_level: c_int);
    pub fn LZ4_loadDictHC(
        ptr: *mut LZ4StreamHC,
        dictionary: *const c_char,
        dict_size: c_int,
    ) -> c_int;
    pub fn LZ4_compress_HC_continue(
        ptr: *mut LZ4StreamHC,
        src: *const c_char,
        dst: *mut c_char,
        src_size: c_int,
        dst_capacity: c_int,
    ) -> c_int;
    pub fn LZ4_compress_HC_continue_destSize(
        ptr: *mut LZ4StreamHC,
        src: *const c_char,
        dst: *mut c_char,
        src_size_ptr: *mut c_int,
        target_dst_size: c_int,
    ) -> c_int;

    #[cfg(feature = "liblz4-experimental")]
    pub fn LZ4_setCompressionLevel(ptr: *mut LZ4StreamHC, compression_level: c_int);

    #[cfg(feature = "liblz4-experimental")]
    pub fn LZ4_favorDecompressionSpeed(ptr: *mut LZ4StreamHC, favor: c_int);
}
