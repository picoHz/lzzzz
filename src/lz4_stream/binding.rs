#![allow(unsafe_code)]

use libc::{c_char, c_int, c_void};

const LZ4_MEMORY_USAGE: usize = 14;
const LZ4_MEMORY_SIZE_U64: usize = (1 << (LZ4_MEMORY_USAGE - 3)) + 4;

#[repr(C)]
pub struct LZ4Stream {
    _private: [u64; LZ4_MEMORY_SIZE_U64],
}

#[repr(C)]
pub struct LZ4DecodeStream {
    _private: [u8; 0],
}

#[link(name = "lz4")]
extern "C" {
    pub fn LZ4_createStream() -> *mut LZ4Stream;
    pub fn LZ4_freeStream(ptr: *mut LZ4Stream) -> c_int;
    pub fn LZ4_resetStream_fast(ptr: *mut LZ4Stream);

    pub fn LZ4_createStreamDecode() -> *mut LZ4DecodeStream;
    pub fn LZ4_freeStreamDecode(ptr: *mut LZ4DecodeStream) -> c_int;
}
