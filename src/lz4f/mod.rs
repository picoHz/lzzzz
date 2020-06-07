//! LZ4 Frame Compressor/Decompressor
//!
//! # Examples
//!
//! Write the compressed `"Hello world!"` to `foo.lz4`.
//!
//! ```
//! use lzzzz::lz4f::FrameCompressor;
//! use std::{fs::File, io::prelude::*};
//!
//! fn main() -> std::io::Result<()> {
//!     let mut output = File::create("foo.lz4")?;
//!     let mut comp = FrameCompressor::new(&mut output)?;
//!
//!     writeln!(comp, "Hello world!")
//! }
//! ```
//!
//! Read and compress data from a slice.
//!
//! ```
//! use lzzzz::lz4f::FrameCompressor;
//! use std::io::prelude::*;
//!
//! fn main() -> std::io::Result<()> {
//!     let input = b"Goodnight world!";
//!     let mut comp = FrameCompressor::new(&input[..])?;
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
//! use lzzzz::lz4f::{BlockSize, FrameCompressorBuilder};
//! use rayon::prelude::*;
//! use std::io::prelude::*;
//!
//! let builder = FrameCompressorBuilder::new().block_size(BlockSize::Max1MB);
//! let all_ok = (1..100)
//!     .into_par_iter()
//!     .map(|n| format!("{} 🐑...", n))
//!     .map_with(builder, |b, data| -> std::io::Result<_> {
//!         let mut buffer = Vec::new();
//!         b.build(data.as_bytes())?.read_to_end(&mut buffer)?;
//!         Ok(buffer)
//!     })
//!     .all(|r| r.is_ok());
//!
//! assert!(all_ok);
//! ```

mod api;
mod binding;

use crate::{lz4f::api::FrameType, Result};
use api::{CompressionContext, DictionaryHandle, LZ4Buffer};
use libc::{c_int, c_uint, c_ulonglong};
use std::{cmp, io, ops, sync::Arc};

/// Compression block size
///
/// **Cited from lz4frame.h:**
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
/// **Cited from lz4frame.h:**
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
/// **Cited from lz4frame.h:**
/// 1: always flush; reduces usage of internal buffers
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum AutoFlush {
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
    Disabled = 0,
    Enabled = 1,
}

/// Frame parameters
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

/// A builder struct to customize `FrameCompressor<D>`.
#[derive(Default, Clone)]
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

    /// Set the compression level.
    ///
    /// **Cited from lz4frame.h:**
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

    /// Create a new `FrameCompressor<D>` instance with this configuration.
    ///
    /// To make I/O operations to the returned `FrameCompressor<D>`,
    /// the `device` should implement `Read`, `BufRead` or `Write`.
    pub fn build<D>(&self, device: D) -> Result<FrameCompressor<D>> {
        FrameCompressor::with_pref(device, self.pref, self.dict.clone())
    }
}

enum CompressorState<D> {
    Created,
    WriteActive {
        finalizer: fn(&mut FrameCompressor<D>) -> Result<()>,
    },
    WriteFinalized,
    ReadActive {
        buffered: ops::Range<usize>,
    },
}

/// The `FrameCompressor<D>` provides a transparent compression to any reader and writer.
///
/// If the underlying I/O device `D` implements `Read`, `BufRead` or `Write`,
/// the `FrameCompressor<D>` also implements `Read`, `BufRead` or `Write`.
///
/// Note that this doesn't mean "Bidirectional stream".
/// Making read and write operations on a same instance causes a panic!
pub struct FrameCompressor<D> {
    pref: Preferences,
    ctx: CompressionContext,
    device: D,
    state: CompressorState<D>,
    buffer: LZ4Buffer,
}

impl<D> FrameCompressor<D> {
    /// Create a new `FrameCompressor<D>` instance with the default configuration.
    pub fn new(device: D) -> Result<Self> {
        Self::with_pref(device, Preferences::default(), None)
    }

    fn with_pref(device: D, pref: Preferences, dict: Option<Dictionary>) -> Result<Self> {
        Ok(Self {
            pref,
            ctx: CompressionContext::new(dict)?,
            device,
            state: CompressorState::Created,
            buffer: LZ4Buffer::new(),
        })
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
        if let CompressorState::WriteActive { .. } = &self.state {
            self.state = CompressorState::WriteFinalized;
            let len = self.ctx.end(&mut self.buffer, None)?;
            self.device.write_all(&self.buffer[..len])?;
            self.device.flush()?;
        }
        Ok(())
    }

    fn ensure_write(&self) {
        if let CompressorState::ReadActive { .. } = self.state {
            panic!("Read operations are not permitted")
        }
    }
}

impl<D: io::Write> io::Write for FrameCompressor<D> {
    fn write(&mut self, src: &[u8]) -> io::Result<usize> {
        self.ensure_write();
        self.buffer.grow(src.len(), Some(&self.pref));
        if let CompressorState::Created = self.state {
            self.state = CompressorState::WriteActive {
                finalizer: FrameCompressor::<D>::finalize_write,
            };
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
            CompressorState::WriteActive { .. } | CompressorState::WriteFinalized => {
                panic!("Write operations are not permitted")
            }
            _ => (),
        }
    }
}

impl<D: io::Read> io::Read for FrameCompressor<D> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.ensure_read();

        let header_len = if let CompressorState::Created = self.state {
            self.state = CompressorState::ReadActive { buffered: 0..0 };
            self.buffer.grow(0, Some(&self.pref));
            self.ctx.begin(&mut self.buffer, Some(&self.pref))?
        } else if let CompressorState::ReadActive { buffered } = &self.state {
            let len = buffered.end - buffered.start;
            let min_len = cmp::min(len, buf.len());
            buf[..min_len].copy_from_slice(&self.buffer[buffered.start..buffered.start + min_len]);
            self.state = CompressorState::ReadActive {
                buffered: if min_len < len {
                    buffered.start + min_len..buffered.end
                } else {
                    0..0
                },
            };
            return Ok(min_len);
        } else {
            0
        };

        let mut tmp = [0u8; 2048];
        let len = self.device.read(&mut tmp[..])?;
        self.buffer.grow(len, Some(&self.pref));

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
            self.state = CompressorState::ReadActive {
                buffered: min_len..len,
            };
        }
        Ok(min_len)
    }
}

impl<D: io::BufRead> io::BufRead for FrameCompressor<D> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        use std::io::Read;
        self.read(&mut [])?;
        if let CompressorState::ReadActive { buffered } = &self.state {
            Ok(&self.buffer[buffered.clone()])
        } else {
            Ok(&[])
        }
    }

    fn consume(&mut self, amt: usize) {
        self.ensure_read();
        if let CompressorState::ReadActive { buffered } = &self.state {
            let len = buffered.end - buffered.start;
            self.state = CompressorState::ReadActive {
                buffered: if amt >= len {
                    0..0
                } else {
                    buffered.start + amt..buffered.end
                },
            };
        }
    }
}

impl<D> Drop for FrameCompressor<D> {
    fn drop(&mut self) {
        let finalizer = if let CompressorState::WriteActive { finalizer } = &self.state {
            finalizer
        } else {
            return;
        };
        let _ = (finalizer)(self);
    }
}

pub fn max_compressed_size(src_size: usize) -> usize {
    0
}

pub fn compress_to_slice(src: &[u8], dst: &mut [u8], preferences: Preferences) -> Result<usize> {
    todo!();
}

/// Compress a buffer into a `Vec<u8>`
///
/// # Examples
///
/// ```
/// use lzzzz::lz4f;
///
/// let mut buf = Vec::new();
/// lz4f::compress(b"Hello world!", &mut buf, lz4f::Preferences::default());
/// ```
pub fn compress(src: &[u8], dst: &mut Vec<u8>, preferences: Preferences) -> Result<()> {
    /*
    use std::io::Write;
    let mut writer = FrameCompressorBuilder::new()
        .compression_level(compression_level)
        .build(dst)?;
    writer.write_all(src)?;
    writer.end()?;
    */
    Ok(())
}

pub fn decompress(src: &[u8], dst: &mut Vec<u8>) -> Result<()> {
    todo!();
}

enum DecompressorState {
    Created,
}

/// The `FrameCompressor<D>` provides a transparent decompression to any reader and writer.
pub struct FrameDecompressor<'a, D> {
    device: D,
    state: DecompressorState,
    dict: &'a [u8],
}

impl<'a, D> FrameDecompressor<'a, D> {
    pub fn new(device: D) -> Result<Self> {
        Ok(Self {
            device,
            state: DecompressorState::Created,
            dict: &[],
        })
    }

    pub fn with_dict(device: D, dict: &'a [u8]) -> Result<Self> {
        Ok(Self {
            device,
            state: DecompressorState::Created,
            dict,
        })
    }

    pub fn set_dict(&mut self, dict: &'a [u8]) {
        self.dict = dict;
    }

    pub fn frame_info(&mut self) -> Result<FrameInfo> {
        todo!();
    }
}

/// A user-defined dictionary for the efficient compression.
///
/// **Cited from lz4frame.h:**
///
/// A Dictionary is useful for the compression of small messages (KB range).
/// It dramatically improves compression efficiency.
///
/// LZ4 can ingest any input as dictionary, though only the last 64 KB are useful.
/// Best results are generally achieved by using Zstandard's Dictionary Builder
/// to generate a high-quality dictionary from a set of samples.
#[derive(Clone)]
pub struct Dictionary(Arc<DictionaryHandle>);

impl Dictionary {
    pub fn new(data: &[u8]) -> Self {
        Self(Arc::new(DictionaryHandle::new(data)))
    }
}

#[cfg(test)]
mod tests {
    use super::{CompressionLevel, Dictionary, FrameCompressorBuilder, Preferences};
    use rand::{distributions::Standard, rngs::SmallRng, Rng, SeedableRng};
    use rayon::prelude::*;
    use std::io::prelude::*;

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
                let src: Vec<_> = rng.sample_iter(Standard).take(n).collect();
                let mut dst = Vec::new();
                super::compress(&src, &mut dst, Preferences::default())
            })
            .all(|r| r.is_ok());
        assert!(all_ok);
    }

    #[test]
    fn parallel_compression_with_dict() {
        let rng = SmallRng::seed_from_u64(0);
        let data: Vec<_> = rng.sample_iter(Standard).take(2048).collect();
        let dict = Dictionary::new(&data);

        let builder = FrameCompressorBuilder::new().dictionary(dict, 1);
        let all_ok = (0..4095usize)
            .into_par_iter()
            .map(|n| {
                let rng = SmallRng::seed_from_u64(n as u64);
                rng.sample_iter(Standard).take(n).collect::<Vec<_>>()
            })
            .map_with(builder, |b, data| -> std::io::Result<_> {
                let mut buffer = Vec::new();
                b.build(data.as_slice())?.read_to_end(&mut buffer)?;
                Ok(buffer)
            })
            .all(|r| r.is_ok());
        assert!(all_ok);
    }

    #[test]
    fn bufread() {
        use crate::lz4f::{BlockSize, FrameCompressorBuilder};
        use std::io::{prelude::*, BufReader};

        fn main() -> std::io::Result<()> {
            let input = b"Goodnight world!";
            let reader = BufReader::new(&input[..]);
            let mut comp = FrameCompressorBuilder::new()
                .block_size(BlockSize::Max1MB)
                .build(reader)?;

            let mut buffer = Vec::new();
            comp.read_until(b'-', &mut buffer)?;
            Ok(())
        }
        main();
    }
}
