#![cfg(feature = "lz4")]
//! Extremely fast compression algorithm

mod binding;
mod block;
mod stream;

pub use block::*;
pub use stream::*;

/// Acceleration mode specifier
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Acceleration {
    /// `Default` is same as `Factor(1)`.
    Default,
    /// Custom acceleration factor.
    ///
    /// Larger value increases the processing speed in exchange for the
    /// loss of compression ratio.
    Factor(i32),
}

impl Default for Acceleration {
    fn default() -> Self {
        Self::Default
    }
}
