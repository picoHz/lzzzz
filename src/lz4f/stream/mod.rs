//! LZ4 Frame Streaming Compressor/Decompressor

pub mod comp;
pub mod decomp;

use crate::common::DEFAULT_BUF_SIZE;
use crate::{
    lz4f::{
        AutoFlush, BlockChecksum, BlockMode, BlockSize, CompressionLevel, ContentChecksum,
        Dictionary, FavorDecSpeed, Preferences,
    },
    Result,
};
use std::convert::TryInto;

/// A builder struct to create a streaming compressor
pub struct CompressorBuilder<D> {
    device: D,
    pref: Preferences,
    dict: Option<Dictionary>,
}

impl<D> CompressorBuilder<D> {
    pub fn new(device: D) -> Self {
        Self {
            device,
            pref: Default::default(),
            dict: None,
        }
    }

    /// Set the compression preferences.
    pub const fn preferences(mut self, pref: Preferences) -> Self {
        self.pref = pref;
        self
    }

    /// Set the block size.
    pub fn block_size(mut self, block_size: BlockSize) -> Self {
        self.pref.set_block_size(block_size);
        self
    }

    /// Set the block mode.
    pub fn block_mode(mut self, block_mode: BlockMode) -> Self {
        self.pref.set_block_mode(block_mode);
        self
    }

    /// Set the content checksum.
    pub fn content_checksum(mut self, checksum: ContentChecksum) -> Self {
        self.pref.set_content_checksum(checksum);
        self
    }

    /// Set the dict id.
    pub fn dict_id(mut self, dict_id: u32) -> Self {
        self.pref.set_dict_id(dict_id);
        self
    }

    /// Set the block checksum.
    pub fn block_checksum(mut self, checksum: BlockChecksum) -> Self {
        self.pref.set_block_checksum(checksum);
        self
    }

    /// Set the compression level.
    pub fn compression_level(mut self, level: CompressionLevel) -> Self {
        self.pref.set_compression_level(level);
        self
    }

    /// Set the decompression speed mode flag.
    pub fn favor_dec_speed(mut self, dec_speed: FavorDecSpeed) -> Self {
        self.pref.set_favor_dec_speed(dec_speed);
        self
    }

    /// Set the auto flush flag.
    pub fn auto_flush(mut self, auto_flush: AutoFlush) -> Self {
        self.pref.set_auto_flush(auto_flush);
        self
    }

    /// Set the compression dictionary.
    pub fn dict(mut self, dict: Dictionary) -> Self {
        self.dict = Some(dict);
        self
    }

    pub fn build<T>(self) -> Result<T>
    where
        Self: TryInto<T, Error = crate::Error>,
    {
        self.try_into()
    }
}

/// A builder struct to create a streaming decompressor
pub struct DecompressorBuilder<D> {
    device: D,
    capacity: usize,
}

impl<D> DecompressorBuilder<D> {
    pub const fn new(device: D) -> Self {
        Self {
            device,
            capacity: DEFAULT_BUF_SIZE,
        }
    }

    /// Set the capacity of the internal buffer.
    pub const fn capacity(mut self, capacity: usize) -> Self {
        self.capacity = capacity;
        self
    }

    pub fn build<T>(self) -> Result<T>
    where
        Self: TryInto<T, Error = crate::Error>,
    {
        self.try_into()
    }
}
