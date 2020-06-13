//! LZ4_HC Compressor
//!
//! The `lz4_hc` module doesn't provide decompression functionalities.
//! Use the [`lz4`] module instead.
//!
//! [`lz4`]: ../lz4/index.html

mod block;
mod stream;

pub use block::*;
pub use stream::*;
