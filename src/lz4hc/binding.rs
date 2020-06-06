#![allow(unsafe_code)]

use libc::{c_char, c_uint, c_void, size_t};

#[link(name = "lz4")]
extern "C" {}
