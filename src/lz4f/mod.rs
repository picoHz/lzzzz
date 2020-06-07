//! LZ4 Frame Compressor/Decompressor
//!
//! # Examples
//!
//! Write the compressed `"Hello world!"` to `foo.lz4`.
//!
//! ```
//! use lzzzz::{lz4f::Preferences, lz4f_stream::StreamCompressor};
//! use std::{fs::File, io::prelude::*};
//!
//! fn main() -> std::io::Result<()> {
//!     let mut output = File::create("foo.lz4")?;
//!     let mut comp = StreamCompressor::new(&mut output, Preferences::default())?;
//!
//!     writeln!(comp, "Hello world!")
//! }
//! ```
//!
//! Read and compress data from a slice.
//!
//! ```
//! use lzzzz::{lz4f::Preferences, lz4f_stream::StreamCompressor};
//! use std::io::prelude::*;
//!
//! fn main() -> std::io::Result<()> {
//!     let input = b"Goodnight world!";
//!     let mut comp = StreamCompressor::new(&input[..], Preferences::default())?;
//!
//!     let mut buffer = Vec::new();
//!     comp.read_to_end(&mut buffer)?;
//!     Ok(())
//! }
//! ```
//!
//! Parallelly count and compress sheep with rayon.
//!
//! ```
//! use lzzzz::{
//!     lz4f::{BlockSize, PreferencesBuilder},
//!     lz4f_stream::StreamCompressor,
//! };
//! use rayon::prelude::*;
//! use std::io::prelude::*;
//!
//! let pref = PreferencesBuilder::new()
//!     .block_size(BlockSize::Max1MB)
//!     .build();
//! let all_ok = (1..100)
//!     .into_par_iter()
//!     .map(|n| format!("{} 🐑...", n))
//!     .map_with(pref, |pref, data| -> std::io::Result<_> {
//!         let mut buffer = Vec::new();
//!         StreamCompressor::new(data.as_bytes(), pref.clone())?.read_to_end(&mut buffer)?;
//!         Ok(buffer)
//!     })
//!     .all(|r| r.is_ok());
//!
//! assert!(all_ok);
//! ```

pub(crate) mod api;
pub(crate) mod binding;

use crate::{lz4f::api::FrameType, Result};
use libc::{c_int, c_uint, c_ulonglong};
use std::{ops, sync::Arc};

/// Compression block size
///
/// **Cited from lz4frame.h:**
/// The larger the block size, the (slightly) better the compression ratio,
/// though there are diminishing returns.
/// Larger blocks also increase memory usage on both compression and
/// decompression sides.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum BlockSize {
    /// Default value
    Default = 0,
    Max64KB = 4,
    Max256KB = 5,
    Max1MB = 6,
    Max4MB = 7,
}

/// Compression block mode
///
/// **Cited from lz4frame.h:**
/// Linked blocks sharply reduce inefficiencies when using small blocks,
/// they compress better.
/// However, some LZ4 decoders are only compatible with independent blocks.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum BlockMode {
    /// Default value
    Linked = 0,
    Independent = 1,
}

/// Compression content checksum
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum ContentChecksum {
    /// Default value
    Disabled = 0,
    Enabled = 1,
}

/// Compression block checksum
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum BlockChecksum {
    /// Default value
    Disabled = 0,
    Enabled = 1,
}

/// Auto flush flag
///
/// **Cited from lz4frame.h:**
/// 1: always flush; reduces usage of internal buffers
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum AutoFlush {
    /// Default value
    Disabled = 0,
    Enabled = 1,
}

/// Decompression speed flag
///
/// **Cited from lz4frame.h:**
/// 1: parser favors decompression speed vs compression ratio.
/// Only works for high compression modes (>= LZ4HC_CLEVEL_OPT_MIN)
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum FavorDecSpeed {
    /// Default value
    Disabled = 0,
    Enabled = 1,
}

/// LZ4 Frame parameters
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct FrameInfo {
    block_size: BlockSize,
    block_mode: BlockMode,
    content_checksum: ContentChecksum,
    frame_type: FrameType,
    content_size: c_ulonglong,
    dict_id: c_uint,
    block_checksum: BlockChecksum,
}

impl Default for FrameInfo {
    fn default() -> Self {
        Self {
            block_size: BlockSize::Default,
            block_mode: BlockMode::Linked,
            content_checksum: ContentChecksum::Disabled,
            frame_type: FrameType::Frame,
            content_size: 0,
            dict_id: 0,
            block_checksum: BlockChecksum::Disabled,
        }
    }
}

impl FrameInfo {
    pub fn block_size(&self) -> BlockSize {
        self.block_size
    }

    pub fn block_mode(&self) -> BlockMode {
        self.block_mode
    }

    pub fn content_checksum(&self) -> ContentChecksum {
        self.content_checksum
    }

    pub fn frame_type(&self) -> FrameType {
        self.frame_type
    }

    pub fn content_size(&self) -> usize {
        self.content_size as usize
    }

    pub fn dict_id(&self) -> u32 {
        self.dict_id as u32
    }

    pub fn block_checksum(&self) -> BlockChecksum {
        self.block_checksum
    }
}

/// Compression level
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CompressionLevel {
    /// Custom compression level.
    /// Values larger then 12 are same as 12. Minus values trigger fast
    /// acceleration.
    Custom(i32),
    /// `Default` is same as `Custom(0)`
    Default,
    /// `High` is same as `Custom(10)`
    High,
    /// `Max` is same as `Custom(12)`
    Max,
}

impl CompressionLevel {
    fn as_i32(self) -> i32 {
        match self {
            Self::Custom(level) => level,
            Self::Default => 0,
            Self::High => 10,
            Self::Max => 12,
        }
    }
}

impl Default for CompressionLevel {
    fn default() -> Self {
        Self::Default
    }
}

/// Compression preferences
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Preferences {
    frame_info: FrameInfo,
    compression_level: c_int,
    auto_flush: AutoFlush,
    favor_dec_speed: FavorDecSpeed,
    _reserved: [c_uint; 3],
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            frame_info: FrameInfo::default(),
            compression_level: 0,
            auto_flush: AutoFlush::Disabled,
            favor_dec_speed: FavorDecSpeed::Disabled,
            _reserved: [0; 3],
        }
    }
}

/// A builder struct to create a custom `Preferences`
///
/// # Example
///
/// ```
/// use lzzzz::lz4f::{BlockSize, CompressionLevel, PreferencesBuilder};
///
/// let pref = PreferencesBuilder::new()
///     .block_size(BlockSize::Max1MB)
///     .compression_level(CompressionLevel::Max)
///     .build();
/// ```
#[derive(Default, Clone)]
pub struct PreferencesBuilder {
    pref: Preferences,
}

impl PreferencesBuilder {
    /// Create a new `PreferencesBuilder` instance with the default
    /// configuration.
    pub fn new() -> Self {
        Default::default()
    }

    /// Set the block size.
    pub fn block_size(mut self, block_size: BlockSize) -> Self {
        self.pref.frame_info.block_size = block_size;
        self
    }

    /// Set the block mode.
    pub fn block_mode(mut self, block_mode: BlockMode) -> Self {
        self.pref.frame_info.block_mode = block_mode;
        self
    }

    /// Set the content checksum.
    pub fn content_checksum(mut self, checksum: ContentChecksum) -> Self {
        self.pref.frame_info.content_checksum = checksum;
        self
    }

    /// Set the dict id.
    pub fn dict_id(mut self, dict_id: u32) -> Self {
        self.pref.frame_info.dict_id = dict_id as u32;
        self
    }

    /// Set the block checksum.
    pub fn block_checksum(mut self, checksum: BlockChecksum) -> Self {
        self.pref.frame_info.block_checksum = checksum;
        self
    }

    /// Set the compression level.
    pub fn compression_level(mut self, level: CompressionLevel) -> Self {
        self.pref.compression_level = level.as_i32() as c_int;
        self
    }

    /// Set the decompression speed mode flag.
    pub fn favor_dec_speed(mut self, dec_speed: FavorDecSpeed) -> Self {
        self.pref.favor_dec_speed = dec_speed;
        self
    }

    /// Set the auto flush flag.
    pub fn auto_flush(mut self, auto_flush: AutoFlush) -> Self {
        self.pref.auto_flush = auto_flush;
        self
    }

    /// Create a new `StreamCompressor<D>` instance with this configuration.
    ///
    /// To make I/O operations to the returned `StreamCompressor<D>`,
    /// the `device` should implement `Read`, `BufRead` or `Write`.
    pub fn build(&self) -> Preferences {
        self.pref.clone()
    }
}

/// Calculate the maximum size of the compressed data from the original size.
pub fn max_compressed_size(uncompressed_size: usize) -> usize {
    0
}

/// Read data from a slice and write compressed data into another slice.
pub fn compress_to_slice(src: &[u8], dst: &mut [u8], preferences: Preferences) -> Result<usize> {
    todo!();
}

/// Read data from a slice and append compressed data to `Vec<u8>`.
///
/// # Examples
///
/// Compress data into the `Vec<u8>` with the default preferences:
/// ```
/// use lzzzz::lz4f;
///
/// let mut buf = Vec::new();
/// lz4f::compress(b"Hello world!", &mut buf, lz4f::Preferences::default());
/// ```
///
/// This function doesn't clear the contents of `Vec<u8>`:
/// ```
/// use lzzzz::lz4f;
///
/// let header = &b"Compressed data:"[..];
/// let mut buf = Vec::from(header);
/// lz4f::compress(b"Hello world!", &mut buf, lz4f::Preferences::default());
/// assert!(buf.starts_with(header));
/// ```
pub fn compress(src: &[u8], dst: &mut Vec<u8>, preferences: Preferences) -> Result<()> {
    // use std::io::Write;
    // let mut writer = PreferencesBuilder::new()
    // .compression_level(compression_level)
    // .build(dst)?;
    // writer.write_all(src)?;
    // writer.end()?;
    Ok(())
}

/// Decompression mode specifier
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DecompressionMode<'a> {
    Default,
    Dictionary { data: &'a [u8] },
}

impl<'a> Default for DecompressionMode<'a> {
    fn default() -> Self {
        Self::Default
    }
}

/// Read data from a slice and write decompressed data into another slice.
pub fn decompress(src: &[u8], dst: &mut Vec<u8>, mode: DecompressionMode) -> Result<()> {
    todo!();
}

// #[cfg(test)]
// mod tests {
// use super::{CompressionLevel, Dictionary, StreamCompressor, Preferences,
// PreferencesBuilder}; use rand::{distributions::Standard, rngs::SmallRng, Rng,
// SeedableRng}; use rayon::prelude::*;
// use std::io::prelude::*;
//
// #[test]
// fn parallel_compression() {
// let all_ok = (0..4095usize)
// .into_par_iter()
// .map(|n| {
// let mut rng = SmallRng::seed_from_u64(n as u64);
// let level = CompressionLevel::Custom(rng.gen_range(
// -CompressionLevel::Max.as_i32(),
// CompressionLevel::Max.as_i32(),
// ));
// let src: Vec<_> = rng.sample_iter(Standard).take(n).collect();
// let mut dst = Vec::new();
// super::compress(&src, &mut dst, Preferences::default())
// })
// .all(|r| r.is_ok());
// assert!(all_ok);
// }
//
// #[test]
// fn parallel_compression_with_dict() {
// let rng = SmallRng::seed_from_u64(0);
// let data: Vec<_> = rng.sample_iter(Standard).take(2048).collect();
// let dict = Dictionary::new(&data);
//
// let pref = PreferencesBuilder::new().dictionary(dict, 1).build();
// let all_ok = (0..4095usize)
// .into_par_iter()
// .map(|n| {
// let rng = SmallRng::seed_from_u64(n as u64);
// rng.sample_iter(Standard).take(n).collect::<Vec<_>>()
// })
// .map_with(pref, |pref, data| -> std::io::Result<_> {
// let mut buffer = Vec::new();
// StreamCompressor::new(data.as_slice(), pref.clone())?.read_to_end(&mut
// buffer)?; Ok(buffer)
// })
// .all(|r| r.is_ok());
// assert!(all_ok);
// }
//
// #[test]
// fn bufread() {
// use crate::lz4f::{BlockSize, StreamCompressor, PreferencesBuilder};
// use std::io::{prelude::*, BufReader};
//
// fn main() -> std::io::Result<()> {
// let input = b"Goodnight world!";
// let reader = BufReader::new(&input[..]);
// let pref = PreferencesBuilder::new()
// .block_size(BlockSize::Max1MB)
// .build();
// let mut comp = StreamCompressor::new(reader, pref)?;
//
// let mut buffer = Vec::new();
// comp.read_until(b'-', &mut buffer)?;
// Ok(())
// }
// main();
// }
// }
