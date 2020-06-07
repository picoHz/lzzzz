#![allow(unsafe_code)]

use libc::{c_char, c_int, c_void, size_t};

const LZ4HC_HASH_LOG: usize = 15;
const LZ4HC_HASHTABLESIZE: usize = 1 << LZ4HC_HASH_LOG;
const LZ4HC_DICTIONARY_LOGSIZE: usize = 16;
const LZ4HC_MAXD: usize = 1 << LZ4HC_DICTIONARY_LOGSIZE;
const LZ4_STREAMHCSIZE: usize = 4 * LZ4HC_HASHTABLESIZE + 2 * LZ4HC_MAXD + 56;
const LZ4_STREAMHCSIZE_SIZET: usize = LZ4_STREAMHCSIZE / std::mem::size_of::<size_t>();

#[repr(C)]
pub struct LZ4StreamHC {
    _private: [size_t; LZ4_STREAMHCSIZE_SIZET],
}

#[link(name = "lz4")]
extern "C" {
    pub fn LZ4_compress_HC(
        src: *const c_char,
        dst: *mut c_char,
        src_size: c_int,
        dst_capacity: c_int,
        compression_level: c_int,
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
}
