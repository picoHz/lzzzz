#![allow(unsafe_code)]

use libc::{c_char, c_int, c_void, size_t};

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
}
