//! Extremely fast compression algorithm.
//!
//! # Acceleration factor
//! Larger value increases the processing speed in exchange for the
//! loss of compression ratio.

mod binding;
mod block;
mod stream;

pub use block::*;
pub use stream::*;

/// Predefined acceleration level (1).
pub const ACC_LEVEL_DEFAULT: i32 = 1;
