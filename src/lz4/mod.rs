#![cfg(feature = "lz4")]
//! LZ4 Compressor/Decompressor

mod block;
mod stream;

pub use block::*;
pub use stream::*;
