#![cfg(feature = "lz4-hc")]
//! High compression variant of LZ4
//!
//! The `lz4_hc` module doesn't provide decompression functionalities.
//! Use the [`lz4`] module instead.
//!
//! [`lz4`]: ../lz4/index.html

mod binding;
mod block;
mod stream;

pub use block::*;
pub use stream::*;

pub const COMPRESSION_LEVEL_MIN: i32 = 3;
pub const COMPRESSION_LEVEL_DEFAULT: i32 = 9;
pub const COMPRESSION_LEVEL_OPTMIN: i32 = 10;
pub const COMPRESSION_LEVEL_MAX: i32 = 12;

/// Decompression speed flag
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
