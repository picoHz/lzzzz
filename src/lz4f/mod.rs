//! LZ4 Frame Compressor/Decompressor

pub mod api;
mod binding;

use crate::Result;
use api::{CompressionContext, Preferences, HEADER_SIZE_MAX};
use libc::{c_int, c_uint, c_ulonglong};
use std::{cmp, io};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
/// Compression block size
///
/// **From lz4frame.h:**
/// The larger the block size, the (slightly) better the compression ratio,
/// though there are diminishing returns.
/// Larger blocks also increase memory usage on both compression and decompression sides.
pub enum BlockSize {
    Default = 0,
    Max64KB = 4,
    Max256KB = 5,
    Max1MB = 6,
    Max4MB = 7,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
/// Compression block mode
///
/// **From lz4frame.h:**
/// Linked blocks sharply reduce inefficiencies when using small blocks,
/// they compress better.
/// However, some LZ4 decoders are only compatible with independent blocks.
pub enum BlockMode {
    Linked = 0,
    Independent = 1,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
/// Compression content checksum
pub enum ContentChecksum {
    Disabled = 0,
    Enabled = 1,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
/// Compression block checksum
pub enum BlockChecksum {
    Disabled = 0,
    Enabled = 1,
}

#[derive(Debug, Default, Copy, Clone)]
/// LZ4 Frame Compressor Builder
pub struct LZ4FrameCompressorBuilder {
    pref: Preferences,
}

impl LZ4FrameCompressorBuilder {
    /// Create a new `LZ4FrameCompressorBuilder` instance with the default configuration.
    pub fn new() -> Self {
        Self {
            pref: Preferences::default(),
        }
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
    pub fn compression_level(mut self, level: i32) -> Self {
        self.pref.compression_level = level as c_int;
        self
    }

    /// Set the dictionary ID.
    pub fn dict_id(mut self, id: u32) -> Self {
        self.pref.frame_info.dict_id = id as c_uint;
        self
    }

    /// Create a new `LZ4FrameCompressor` instance with this configuration.
    pub fn build<W: io::Write>(self, writer: W) -> Result<LZ4FrameCompressor<W>> {
        LZ4FrameCompressor::new(writer, self.pref)
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
/// use lzzzz::lz4f::{BlockSize, LZ4FrameCompressorBuilder};
/// use std::{fs::File, io::prelude::*};
///
/// fn main() -> std::io::Result<()> {
///     let mut output = File::create("foo.lz4")?;
///     let mut comp = LZ4FrameCompressorBuilder::new()
///         .block_size(BlockSize::Max1MB)
///         .build(&mut output)?;
///
///     writeln!(comp, "Hello world!")
/// }
/// ```
pub struct LZ4FrameCompressor<W: io::Write> {
    pref: Preferences,
    ctx: CompressionContext,
    buffer: Vec<u8>,
    writer: W,
    state: State,
    prev_src_size: usize,
}

impl<W: io::Write> LZ4FrameCompressor<W> {
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
    /// Dropping a `LZ4FrameCompressor` automatically finalize a frame
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
            State::Finalized => unreachable!(),
            _ => Ok(()),
        }
    }
}

impl<W: io::Write> io::Write for LZ4FrameCompressor<W> {
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
        self.writer.write_all(&self.buffer[..len])
    }
}

impl<W: io::Write> Drop for LZ4FrameCompressor<W> {
    fn drop(&mut self) {
        let _ = self.finalize();
    }
}
