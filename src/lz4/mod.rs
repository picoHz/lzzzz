#![cfg(feature = "lz4")]
//! Extremely fast compression algorithm

mod binding;
mod block;
mod stream;

pub use block::*;
pub use stream::*;

/// Predefined acceleration level (1)
pub const ACCELERATION_DEFAULT: i32 = 1;
