use std::os::raw::{c_char, c_int};

#[allow(non_camel_case_types)]
type size_t = usize;

#[link(name = "lz4")]
extern "C" {
    pub fn LZ4_versionNumber() -> c_int;
    pub fn LZ4_versionString() -> *const c_char;
}
