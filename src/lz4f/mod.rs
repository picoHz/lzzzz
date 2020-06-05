//! LZ4 Frame Compressor/Decompressor

pub mod api;
mod binding;

use crate::Result;
use api::{CompressionContext, DictionaryHandle, Preferences, HEADER_SIZE_MAX};
use libc::{c_int, c_uint, c_ulonglong};
use std::{cmp, io, sync::Arc};

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
    pub fn build<W: io::Write>(self, writer: W) -> Result<FrameCompressor<W>> {
        FrameCompressor::new(writer, self.pref)
    }
}

enum State {
    Created,
    Active,
    Finalized,
}

/// LZ4 Frame Compressor
///
/// # Examples
///
/// Write the compressed `"Hello world!"` to `foo.lz4`.
///
/// ```
/// use lzzzz::lz4f::{BlockSize, FrameCompressorBuilder};
/// use std::{fs::File, io::prelude::*};
///
/// fn main() -> std::io::Result<()> {
///     let mut output = File::create("foo.lz4")?;
///     let mut comp = FrameCompressorBuilder::new()
///         .block_size(BlockSize::Max1MB)
///         .build(&mut output)?;
///
///     writeln!(comp, "Hello world!")
/// }
/// ```
pub struct FrameCompressor<W: io::Write> {
    pref: Preferences,
    ctx: CompressionContext,
    buffer: Vec<u8>,
    writer: W,
    state: State,
    prev_src_size: usize,
}

impl<W: io::Write> FrameCompressor<W> {
    fn new(writer: W, pref: Preferences) -> Result<Self> {
        Ok(Self {
            pref,
            ctx: CompressionContext::new()?,
            buffer: Vec::new(),
            writer,
            state: State::Created,
            prev_src_size: 0,
        })
    }

    /// Finalize this LZ4 frame explicitly.
    ///
    /// Dropping a `FrameCompressor` automatically finalize a frame
    /// so you don't have to call this unless you need a `Result`.
    pub fn end(mut self) -> Result<()> {
        self.finalize()
    }

    fn grow_buffer(&mut self, src_size: usize) {
        if src_size > self.prev_src_size {
            let len = CompressionContext::compress_bound(src_size, Some(&self.pref));
            let len = cmp::max(len, HEADER_SIZE_MAX);
            if len > self.buffer.len() {
                self.buffer.reserve(len - self.buffer.len());

                #[allow(unsafe_code)]
                unsafe {
                    self.buffer.set_len(len)
                };
            }
            self.prev_src_size = src_size;
        }
    }

    fn finalize(&mut self) -> Result<()> {
        match self.state {
            State::Active => {
                self.state = State::Finalized;
                let len = self.ctx.end(&mut self.buffer, None)?;
                self.writer.write_all(&self.buffer[..len])?;
                self.writer.flush()?;
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

impl<W: io::Write> io::Write for FrameCompressor<W> {
    fn write(&mut self, src: &[u8]) -> io::Result<usize> {
        self.grow_buffer(src.len());
        if let State::Created = self.state {
            self.state = State::Active;
            let len = self.ctx.begin(&mut self.buffer, Some(&self.pref))?;
            self.writer.write(&self.buffer[..len])?;
        }
        let len = self.ctx.update(&mut self.buffer, src, None)?;
        self.writer.write(&self.buffer[..len])?;
        Ok(src.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let len = self.ctx.flush(&mut self.buffer, None)?;
        self.writer.write_all(&self.buffer[..len])?;
        self.writer.flush()
    }
}

impl<W: io::Write> Drop for FrameCompressor<W> {
    fn drop(&mut self) {
        let _ = self.finalize();
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
        .compression_level(compression_level.as_i32())
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
