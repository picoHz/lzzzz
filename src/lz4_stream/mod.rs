//! LZ4 Streaming Compressor/Decompressor

mod api;

pub struct StreamCompressor<D> {
    device: D,
}

impl<D> StreamCompressor<D> {}

pub struct StreamDecompressor<D> {
    device: D,
}

impl<D> StreamDecompressor<D> {}
