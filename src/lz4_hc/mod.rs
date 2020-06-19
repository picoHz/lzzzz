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
