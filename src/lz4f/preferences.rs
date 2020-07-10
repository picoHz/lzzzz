use super::frame_info::{BlockChecksum, BlockMode, BlockSize, ContentChecksum, FrameInfo};
use std::os::raw::{c_int, c_uint};

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

impl Default for AutoFlush {
    fn default() -> Self {
        Self::Disabled
    }
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

impl Default for FavorDecSpeed {
    fn default() -> Self {
        Self::Disabled
    }
}

pub const COMPRESSION_LEVEL_DEFAULT: i32 = 0;
pub const COMPRESSION_LEVEL_HIGH: i32 = 10;
pub const COMPRESSION_LEVEL_MAX: i32 = 12;

/// Compression level specifier
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
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
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

    pub(super) fn set_compression_level(&mut self, level: i32) {
        self.compression_level = level as c_int;
    }

    pub(super) fn set_favor_dec_speed(&mut self, dec_speed: FavorDecSpeed) {
        self.favor_dec_speed = dec_speed;
    }

    pub(super) fn set_auto_flush(&mut self, auto_flush: AutoFlush) {
        self.auto_flush = auto_flush;
    }
}

/// A builder struct to create a custom `Preferences`
///
/// # Example
///
/// ```
/// use lzzzz::lz4f::{BlockSize, PreferencesBuilder, COMPRESSION_LEVEL_MAX};
///
/// let pref = PreferencesBuilder::new()
///     .block_size(BlockSize::Max1MB)
///     .compression_level(COMPRESSION_LEVEL_MAX)
///     .build();
/// ```
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PreferencesBuilder {
    prefs: Preferences,
}

impl PreferencesBuilder {
    /// Create a new `PreferencesBuilder` instance with the default
    /// configuration.
    pub fn new() -> Self {
        Default::default()
    }

    /// Set the block size.
    pub fn block_size(&mut self, block_size: BlockSize) -> &mut Self {
        self.prefs.set_block_size(block_size);
        self
    }

    /// Set the block mode.
    pub fn block_mode(&mut self, block_mode: BlockMode) -> &mut Self {
        self.prefs.set_block_mode(block_mode);
        self
    }

    /// Set the content checksum.
    pub fn content_checksum(&mut self, checksum: ContentChecksum) -> &mut Self {
        self.prefs.set_content_checksum(checksum);
        self
    }

    /// Set the dict id.
    pub fn dict_id(&mut self, dict_id: u32) -> &mut Self {
        self.prefs.set_dict_id(dict_id);
        self
    }

    /// Set the block checksum.
    pub fn block_checksum(&mut self, checksum: BlockChecksum) -> &mut Self {
        self.prefs.set_block_checksum(checksum);
        self
    }

    /// Set the compression level.
    pub fn compression_level(&mut self, level: i32) -> &mut Self {
        self.prefs.set_compression_level(level);
        self
    }

    /// Set the decompression speed mode flag.
    pub fn favor_dec_speed(&mut self, dec_speed: FavorDecSpeed) -> &mut Self {
        self.prefs.set_favor_dec_speed(dec_speed);
        self
    }

    /// Set the auto flush flag.
    pub fn auto_flush(&mut self, auto_flush: AutoFlush) -> &mut Self {
        self.prefs.set_auto_flush(auto_flush);
        self
    }

    /// Create a new `Compressor<D>` instance with this configuration.
    ///
    /// To make I/O operations to the returned `Compressor<D>`,
    /// the `device` should implement `Read`, `BufRead` or `Write`.
    pub const fn build(&self) -> Preferences {
        self.prefs
    }
}

#[cfg(test)]
mod tests {
    use crate::lz4f::{
        BlockChecksum, BlockMode, BlockSize, ContentChecksum, FavorDecSpeed, Preferences,
        PreferencesBuilder, COMPRESSION_LEVEL_DEFAULT, COMPRESSION_LEVEL_HIGH,
        COMPRESSION_LEVEL_MAX,
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
                .compression_level(i32::MAX)
                .build()
                .compression_level,
            i32::MAX
        );
        assert_eq!(
            PreferencesBuilder::new()
                .compression_level(COMPRESSION_LEVEL_DEFAULT)
                .build()
                .compression_level,
            COMPRESSION_LEVEL_DEFAULT
        );
        assert_eq!(
            PreferencesBuilder::new()
                .compression_level(COMPRESSION_LEVEL_HIGH)
                .build()
                .compression_level,
            COMPRESSION_LEVEL_HIGH
        );
        assert_eq!(
            PreferencesBuilder::new()
                .compression_level(COMPRESSION_LEVEL_MAX)
                .build()
                .compression_level,
            COMPRESSION_LEVEL_MAX
        );
        assert_eq!(
            PreferencesBuilder::new()
                .compression_level(i32::MIN)
                .build()
                .compression_level,
            i32::MIN
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
}
