use super::frame_info::{BlockChecksum, BlockMode, BlockSize, ContentChecksum, FrameInfo};
use std::{
    cmp,
    os::raw::{c_int, c_uint},
};

/// Auto flush flag
///
/// **Cited from lz4frame.h:**
/// 1: always flush; reduces usage of internal buffers
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum AutoFlush {
    /// Default value
    Disabled,
    Enabled,
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
    Disabled,
    Enabled,
}

/// Compression level specifier
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, Hash)]
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

impl PartialOrd for CompressionLevel {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.as_i32().cmp(&other.as_i32()))
    }
}

impl CompressionLevel {
    pub(crate) fn as_i32(self) -> i32 {
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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct Preferences {
    frame_info: FrameInfo,
    compression_level: c_int,
    auto_flush: AutoFlush,
    favor_dec_speed: FavorDecSpeed,
    _reserved: [c_uint; 3],
}

impl Preferences {
    /// Return the frame info.
    pub const fn frame_info(&self) -> FrameInfo {
        self.frame_info
    }

    /// Return the compression level.
    pub const fn compression_level(&self) -> i32 {
        self.compression_level as i32
    }

    /// Return the auto flush mode flag.
    pub const fn auto_flush(&self) -> AutoFlush {
        self.auto_flush
    }

    /// Return the decompression speed mode flag.
    pub const fn favor_dec_speed(&self) -> FavorDecSpeed {
        self.favor_dec_speed
    }

    pub(super) fn set_block_size(&mut self, block_size: BlockSize) {
        self.frame_info.set_block_size(block_size);
    }

    pub(super) fn set_block_mode(&mut self, block_mode: BlockMode) {
        self.frame_info.set_block_mode(block_mode);
    }

    pub(super) fn set_content_checksum(&mut self, checksum: ContentChecksum) {
        self.frame_info.set_content_checksum(checksum);
    }

    pub(super) fn set_dict_id(&mut self, dict_id: u32) {
        self.frame_info.set_dict_id(dict_id);
    }

    pub(super) fn set_block_checksum(&mut self, checksum: BlockChecksum) {
        self.frame_info.set_block_checksum(checksum);
    }

    pub(super) fn set_compression_level(&mut self, level: CompressionLevel) {
        // Workaround for the integer overflow bug in liblz4.
        const MIN_COMPRESSION_LEVEL: i32 = -33_554_430;
        self.compression_level = cmp::max(level.as_i32(), MIN_COMPRESSION_LEVEL) as c_int;
    }

    pub(super) fn set_favor_dec_speed(&mut self, dec_speed: FavorDecSpeed) {
        self.favor_dec_speed = dec_speed;
    }

    pub(super) fn set_auto_flush(&mut self, auto_flush: AutoFlush) {
        self.auto_flush = auto_flush;
    }
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
    pub fn block_size(&mut self, block_size: BlockSize) -> &mut Self {
        self.pref.set_block_size(block_size);
        self
    }

    /// Set the block mode.
    pub fn block_mode(&mut self, block_mode: BlockMode) -> &mut Self {
        self.pref.set_block_mode(block_mode);
        self
    }

    /// Set the content checksum.
    pub fn content_checksum(&mut self, checksum: ContentChecksum) -> &mut Self {
        self.pref.set_content_checksum(checksum);
        self
    }

    /// Set the dict id.
    pub fn dict_id(&mut self, dict_id: u32) -> &mut Self {
        self.pref.set_dict_id(dict_id);
        self
    }

    /// Set the block checksum.
    pub fn block_checksum(&mut self, checksum: BlockChecksum) -> &mut Self {
        self.pref.set_block_checksum(checksum);
        self
    }

    /// Set the compression level.
    pub fn compression_level(&mut self, level: CompressionLevel) -> &mut Self {
        self.pref.set_compression_level(level);
        self
    }

    /// Set the decompression speed mode flag.
    pub fn favor_dec_speed(&mut self, dec_speed: FavorDecSpeed) -> &mut Self {
        self.pref.set_favor_dec_speed(dec_speed);
        self
    }

    /// Set the auto flush flag.
    pub fn auto_flush(&mut self, auto_flush: AutoFlush) -> &mut Self {
        self.pref.set_auto_flush(auto_flush);
        self
    }

    /// Create a new `Compressor<D>` instance with this configuration.
    ///
    /// To make I/O operations to the returned `Compressor<D>`,
    /// the `device` should implement `Read`, `BufRead` or `Write`.
    pub const fn build(&self) -> Preferences {
        self.pref
    }
}

#[cfg(test)]
mod tests {
    use crate::lz4f::{
        BlockChecksum, BlockMode, BlockSize, CompressionLevel, ContentChecksum, FavorDecSpeed,
        Preferences, PreferencesBuilder,
    };
    use std::{i32, u32};

    #[test]
    fn preferences_builder() {
        assert_eq!(PreferencesBuilder::new().build(), Preferences::default());
        assert_eq!(
            PreferencesBuilder::new()
                .favor_dec_speed(FavorDecSpeed::Enabled)
                .build()
                .favor_dec_speed,
            FavorDecSpeed::Enabled
        );
        assert_eq!(
            PreferencesBuilder::new()
                .block_size(BlockSize::Max64KB)
                .build()
                .frame_info
                .block_size(),
            BlockSize::Max64KB
        );
        assert_eq!(
            PreferencesBuilder::new()
                .block_size(BlockSize::Max256KB)
                .build()
                .frame_info
                .block_size(),
            BlockSize::Max256KB
        );
        assert_eq!(
            PreferencesBuilder::new()
                .block_size(BlockSize::Max1MB)
                .build()
                .frame_info
                .block_size(),
            BlockSize::Max1MB
        );
        assert_eq!(
            PreferencesBuilder::new()
                .block_size(BlockSize::Max4MB)
                .build()
                .frame_info
                .block_size(),
            BlockSize::Max4MB
        );
        assert_eq!(
            PreferencesBuilder::new()
                .content_checksum(ContentChecksum::Enabled)
                .build()
                .frame_info
                .content_checksum(),
            ContentChecksum::Enabled
        );
        assert_eq!(
            PreferencesBuilder::new()
                .block_mode(BlockMode::Independent)
                .build()
                .frame_info
                .block_mode(),
            BlockMode::Independent
        );
        assert_eq!(
            PreferencesBuilder::new()
                .compression_level(CompressionLevel::Custom(i32::MAX))
                .build()
                .compression_level,
            CompressionLevel::Custom(i32::MAX).as_i32()
        );
        assert_eq!(
            PreferencesBuilder::new()
                .compression_level(CompressionLevel::High)
                .build()
                .compression_level,
            CompressionLevel::High.as_i32()
        );
        assert_eq!(
            PreferencesBuilder::new()
                .compression_level(CompressionLevel::Max)
                .build()
                .compression_level,
            CompressionLevel::Max.as_i32()
        );
        assert_eq!(
            PreferencesBuilder::new()
                .compression_level(CompressionLevel::Custom(i32::MIN))
                .build()
                .compression_level,
            -33_554_430
        );
        assert_eq!(
            PreferencesBuilder::new()
                .block_checksum(BlockChecksum::Enabled)
                .build()
                .frame_info
                .block_checksum(),
            BlockChecksum::Enabled
        );
        assert_eq!(
            PreferencesBuilder::new()
                .dict_id(u32::MAX)
                .build()
                .frame_info
                .dict_id(),
            u32::MAX
        );
        assert_eq!(
            PreferencesBuilder::new()
                .dict_id(u32::MIN)
                .build()
                .frame_info
                .dict_id(),
            u32::MIN
        );
    }

    #[test]
    fn compression_level() {
        assert_eq!(CompressionLevel::Default, CompressionLevel::default());
        assert_eq!(
            CompressionLevel::Default.as_i32(),
            CompressionLevel::Custom(0).as_i32()
        );
        assert_eq!(
            CompressionLevel::High.as_i32(),
            CompressionLevel::Custom(10).as_i32()
        );
        assert_eq!(
            CompressionLevel::Max.as_i32(),
            CompressionLevel::Custom(12).as_i32()
        );

        let mut sorted = vec![
            CompressionLevel::Custom(i32::MAX),
            CompressionLevel::Max,
            CompressionLevel::High,
            CompressionLevel::Default,
            CompressionLevel::Custom(i32::MIN),
        ];
        sorted.sort_unstable();
        assert_eq!(
            sorted,
            vec![
                CompressionLevel::Custom(i32::MIN),
                CompressionLevel::Default,
                CompressionLevel::High,
                CompressionLevel::Max,
                CompressionLevel::Custom(i32::MAX),
            ]
        );
    }
}
