//! LZ4 Frame Compressor/Decompressor
//!
//! # Examples
//!
//! Write the compressed `"Hello world!"` to `foo.lz4`.
//!
//! ```
//! use lzzzz::lz4f::{BlockSize, FrameCompressorBuilder};
//! use std::{fs::File, io::prelude::*};
//!
//! fn main() -> std::io::Result<()> {
//!     let mut output = File::create("foo.lz4")?;
//!     let mut comp = FrameCompressorBuilder::new()
//!         .block_size(BlockSize::Max1MB)
//!         .build(&mut output)?;
//!
//!     writeln!(comp, "Hello world!")
//! }
//! ```
//!
//! Read and compress data from a slice.
//!
//! ```
//! use lzzzz::lz4f::{BlockSize, FrameCompressorBuilder};
//! use std::io::prelude::*;
//!
//! fn main() -> std::io::Result<()> {
//!     let input = b"Hello world!";
//!     let mut comp = FrameCompressorBuilder::new()
//!         .block_size(BlockSize::Max1MB)
//!         .build(&input[..])?;
//!
//!     let mut buffer = Vec::new();
//!     comp.read_to_end(&mut buffer)?;
//!     Ok(())
//! }
//! ```
//!
//! Parallel compression with rayon.
//!
//! ```
//! use lzzzz::lz4f::{BlockSize, FrameCompressorBuilder};
//! use rayon::prelude::*;
//! use std::io::prelude::*;
//!
//! let input: Vec<_> = (0..5).map(|n| format!("Hello world {}!", n)).collect();
//! let builder = FrameCompressorBuilder::new().block_size(BlockSize::Max1MB);
//!
//! let result: Vec<std::io::Result<_>> = input
//!     .into_par_iter()
//!     .map_with(builder, |b, data| {
//!         let mut buffer = Vec::new();
//!         b.build(data.as_bytes())?.read_to_end(&mut buffer)?;
//!         Ok(buffer)
//!     })
//!     .collect();
//!
//! println!("{:?}", result);
//! ```

pub mod api;
mod binding;

use crate::{LZ4Error, Result};
use api::{CompressionContext, DictionaryHandle, Preferences, HEADER_SIZE_MAX};
use libc::{c_int, c_uint, c_ulonglong};
use std::{cmp, io, ops, sync::Arc};

/// Compression block size
///
/// **From lz4frame.h:**
/// The larger the block size, the (slightly) better the compression ratio,
/// though there are diminishing returns.
/// Larger blocks also increase memory usage on both compression and decompression sides.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum BlockSize {
    Default = 0,
    Max64KB = 4,
    Max256KB = 5,
    Max1MB = 6,
    Max4MB = 7,
}

/// Compression block mode
///
/// **From lz4frame.h:**
/// Linked blocks sharply reduce inefficiencies when using small blocks,
/// they compress better.
/// However, some LZ4 decoders are only compatible with independent blocks.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum BlockMode {
    Linked = 0,
    Independent = 1,
}

/// Compression content checksum
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum ContentChecksum {
    Disabled = 0,
    Enabled = 1,
}

/// Compression block checksum
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum BlockChecksum {
    Disabled = 0,
    Enabled = 1,
}

/// Auto flush flag
///
/// **From lz4frame.h:**
/// 1: always flush; reduces usage of internal buffers
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum AutoFlush {
    Disabled = 0,
    Enabled = 1,
}

/// Decompression speed flag
///
/// **From lz4frame.h:**
/// 1: parser favors decompression speed vs compression ratio.
/// Only works for high compression modes (>= LZ4HC_CLEVEL_OPT_MIN)
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum FavorDecSpeed {
    Disabled = 0,
    Enabled = 1,
}

#[derive(Default, Clone)]
/// LZ4 Frame Compressor Builder
pub struct FrameCompressorBuilder {
    pref: Preferences,
    dict: Option<Dictionary>,
}

impl FrameCompressorBuilder {
    /// Create a new `FrameCompressorBuilder` instance with the default configuration.
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

    /// Set the block checksum.
    pub fn block_checksum(mut self, checksum: BlockChecksum) -> Self {
        self.pref.frame_info.block_checksum = checksum;
        self
    }

    /// Set the size of uncompressed content.
    pub fn content_size(mut self, size: usize) -> Self {
        self.pref.frame_info.content_size = size as c_ulonglong;
        self
    }

    /// Set the compression level.
    ///
    /// **From lz4frame.h:**
    /// 0: default (fast mode); values > LZ4HC_CLEVEL_MAX count as LZ4HC_CLEVEL_MAX;
    /// values < 0 trigger "fast acceleration"
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

    /// Set the dictionary and the dictionary ID.
    pub fn dictionary(mut self, dict: Dictionary, dict_id: u32) -> Self {
        self.pref.frame_info.dict_id = dict_id as c_uint;
        self.dict = Some(dict);
        self
    }

    /// Create a new `FrameCompressor` instance with this configuration.
    ///
    /// The `device` should implement either `Read` or `Write`,
    /// or the returned `FrameCompressor` become useless.
    pub fn build<D>(&self, device: D) -> Result<FrameCompressor<D>> {
        FrameCompressor::new(device, self.pref)
    }
}

enum State {
    Created,
    WriteActive,
    WriteFinalized,
    ReadActive,
}

/// LZ4 Frame Compressor
pub struct FrameCompressor<D> {
    pref: Preferences,
    ctx: CompressionContext,
    buffer: Vec<u8>,
    device: D,
    state: State,
    prev_size: usize,
    buffered: ops::Range<usize>,
    finalizer: Option<fn(&mut Self) -> Result<()>>,
}

impl<D> FrameCompressor<D> {
    fn new(device: D, pref: Preferences) -> Result<Self> {
        Ok(Self {
            pref,
            ctx: CompressionContext::new()?,
            buffer: Vec::new(),
            device,
            state: State::Created,
            prev_size: 0,
            buffered: 0..0,
            finalizer: None,
        })
    }

    fn grow_buffer(&mut self, src_size: usize) {
        if src_size == 0 || src_size > self.prev_size {
            let len =
                CompressionContext::compress_bound(src_size + HEADER_SIZE_MAX, Some(&self.pref));
            if len > self.buffer.len() {
                self.buffer.reserve(len - self.buffer.len());

                #[allow(unsafe_code)]
                unsafe {
                    self.buffer.set_len(len)
                };
            }
            self.prev_size = src_size;
        }
    }
}

impl<D: io::Write> FrameCompressor<D> {
    /// Finalize this LZ4 frame explicitly.
    ///
    /// Dropping a `FrameCompressor` automatically finalize a frame
    /// so you don't have to call this unless you need a `Result`.
    pub fn end(mut self) -> Result<()> {
        self.finalize_write()
    }

    fn finalize_write(&mut self) -> Result<()> {
        self.ensure_write();
        match self.state {
            State::WriteActive => {
                self.state = State::WriteFinalized;
                let len = self.ctx.end(&mut self.buffer, None)?;
                self.device.write_all(&self.buffer[..len])?;
                self.device.flush()?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn ensure_write(&self) {
        match self.state {
            State::ReadActive => panic!("Read operations are not permitted"),
            _ => (),
        }
    }
}

impl<D: io::Write> io::Write for FrameCompressor<D> {
    fn write(&mut self, src: &[u8]) -> io::Result<usize> {
        self.ensure_write();
        self.grow_buffer(src.len());
        if let State::Created = self.state {
            self.finalizer = Some(Self::finalize_write);
            self.state = State::WriteActive;
            let len = self.ctx.begin(&mut self.buffer, Some(&self.pref))?;
            self.device.write(&self.buffer[..len])?;
        }
        let len = self.ctx.update(&mut self.buffer, src, None)?;
        self.device.write(&self.buffer[..len])?;
        Ok(src.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.ensure_write();
        let len = self.ctx.flush(&mut self.buffer, None)?;
        self.device.write_all(&self.buffer[..len])?;
        self.device.flush()
    }
}

impl<D: io::Read> FrameCompressor<D> {
    fn ensure_read(&self) {
        match self.state {
            State::WriteActive | State::WriteFinalized => {
                panic!("Write operations are not permitted")
            }
            _ => (),
        }
    }
}

impl<D: io::Read> io::Read for FrameCompressor<D> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.ensure_read();

        let len = self.buffered.end - self.buffered.start;
        if len > 0 {
            let min_len = cmp::min(len, buf.len());
            buf[..min_len]
                .copy_from_slice(&self.buffer[self.buffered.start..self.buffered.start + min_len]);
            self.buffered = if min_len < len {
                self.buffered.start + min_len..self.buffered.end
            } else {
                0..0
            };
            return Ok(min_len);
        }

        let mut tmp = [0u8; 2048];
        let header_len = if let State::Created = self.state {
            self.state = State::ReadActive;
            self.grow_buffer(0);
            self.ctx.begin(&mut self.buffer, Some(&self.pref))?
        } else {
            0
        };
        let len = self.device.read(&mut tmp[..])?;
        self.grow_buffer(len);

        let len = if len == 0 {
            self.ctx.flush(&mut self.buffer[header_len..], None)?
        } else {
            self.ctx
                .update(&mut self.buffer[header_len..], &tmp[..len], None)?
        };
        let len = header_len + len;
        let min_len = cmp::min(len, buf.len());
        buf[..min_len].copy_from_slice(&self.buffer[..min_len]);
        if min_len < len {
            self.buffered = min_len..len;
        }
        Ok(min_len)
    }
}

impl<D> Drop for FrameCompressor<D> {
    fn drop(&mut self) {
        if let Some(finalizer) = self.finalizer {
            let _ = (finalizer)(self);
        }
    }
}

/// Compression level.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CompressionLevel {
    Custom(i32),
    Default,
    High,
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

/// Compress a buffer into a `Vec<u8>`
///
/// # Examples
///
/// ```
/// use lzzzz::lz4f;
///
/// let compressed = lz4f::compress(b"Hello world!", lz4f::CompressionLevel::Default);
/// println!("{:?}", compressed);
/// ```
pub fn compress(data: &[u8], compression_level: CompressionLevel) -> Result<Vec<u8>> {
    use std::io::Write;
    let mut buf = Vec::new();
    let mut comp = FrameCompressorBuilder::new()
        .compression_level(compression_level)
        .build(&mut buf)?;
    comp.write_all(data)?;
    comp.end()?;
    Ok(buf)
}

/// Dictionary
///
/// A `Dictionary` can be shared by multiple threads safely.
#[derive(Clone)]
pub struct Dictionary(Arc<DictionaryHandle>);

impl Dictionary {
    pub fn new(data: &[u8]) -> Self {
        Self(Arc::new(DictionaryHandle::new(data)))
    }
}

#[cfg(test)]
mod tests {
    use super::CompressionLevel;
    use rand::{distributions::Standard, rngs::SmallRng, Rng, SeedableRng};
    use rayon::prelude::*;

    #[test]
    fn parallel_compression() {
        let all_ok = (0..4095usize)
            .into_par_iter()
            .map(|n| {
                let mut rng = SmallRng::seed_from_u64(n as u64);
                let level = CompressionLevel::Custom(rng.gen_range(
                    -CompressionLevel::Max.as_i32(),
                    CompressionLevel::Max.as_i32(),
                ));
                let data: Vec<u8> = rng.sample_iter(Standard).take(n).collect();
                super::compress(&data, level)
            })
            .all(|r| r.is_ok());
        assert!(all_ok);
    }
}
