use std::os::raw::{c_char, c_int, c_uint};

#[allow(non_camel_case_types)]
type size_t = usize;

#[link(name = "lz4")]
extern "C" {
    pub fn LZ4_versionNumber() -> c_int;
    pub fn LZ4_versionString() -> *const c_char;
    pub fn LZ4F_isError(code: size_t) -> c_uint;
}
