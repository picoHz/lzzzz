#![cfg(feature = "lz4f")]
//! LZ4 Frame Compressor/Decompressor

mod frame;
mod stream;

pub use frame::*;
pub use stream::*;

pub use stream::{CompressorBuilder, DecompressorBuilder};
