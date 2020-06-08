//! Yet another liblz4 binding supports verious LZ4 operations

#![deny(unsafe_code)]

mod binding;
mod common;
pub mod lz4;
pub mod lz4_hc;
pub mod lz4_hc_stream;
pub mod lz4_stream;
pub mod lz4f;
pub mod lz4f_stream;

pub use common::{version_number, version_string, LZ4Error, Result};
