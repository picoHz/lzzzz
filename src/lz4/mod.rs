//! Extremely fast compression algorithm.
//!
//! # Block mode
//!
//! # Streaming mode
//!
//! # Acceleration factor
//! Larger value increases the processing speed in exchange for the
//! loss of compression ratio.
//! `ACC_LEVEL_DEFAULT`

mod binding;
mod block;
mod stream;

pub use block::*;
pub use stream::*;

/// Predefined acceleration level (1)
pub const ACC_LEVEL_DEFAULT: i32 = 1;
