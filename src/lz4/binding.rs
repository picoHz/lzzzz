#![allow(unsafe_code)]

use libc::{c_char, c_int, c_void};

#[link(name = "lz4")]
extern "C" {
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
}
