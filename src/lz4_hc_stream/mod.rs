//! LZ4_HC Streaming Compressor
//!
//! The `lz4_hc_stream` module doesn't provide decompression functionalities.
//! Use the [`lz4_stream`] module instead.
//!
//! [`lz4_stream`]: ../lz4_stream/index.html

mod api;

use crate::lz4_hc::CompressionLevel;

pub struct StreamCompressor<D> {
    device: D,
}

impl<D> StreamCompressor<D> {
    pub fn set_compression_level(&mut self, level: &CompressionLevel) {}

    pub fn set_favor_dec_speed(&mut self, flag: bool) {}
}
