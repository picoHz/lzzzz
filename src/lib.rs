//! Yet another liblz4 binding 😴
//!
//! - **Designed for Rust:** Lzzzz is a high-level wrapper of liblz4 and
//! provides comprehensible API complies with Rust's manner without losing performance and
//! flexibility. You have no concern about memory management and concurrency
//! safety.
//!
//! - **Various Supported Modes:** [`LZ4`], [`LZ4_HC`], [`LZ4F`], [`LZ4 Streaming`],
//! [`LZ4_HC Streaming`] and [`LZ4F Streaming`] are supported.
//!
//! - **Flexible Streaming:** Compressor/Decompressor stream supports [`Read`],
//! [`BufRead`] and [`Write`] operations. With `tokio` feature, `AsyncRead`,
//! `AsyncBufRead` and `AsyncWrite` are also supported.
//!
//! [`LZ4`]: ./lz4/index.html
//! [`LZ4_HC`]: ./lz4_hc/index.html
//! [`LZ4F`]: ./lz4f/index.html
//! [`LZ4 Streaming`]: ./lz4_stream/index.html
//! [`LZ4_HC Streaming`]: ./lz4_hc_stream/index.html
//! [`LZ4F Streaming`]: ./lz4f_stream/index.html
//! [`Read`]: https://doc.rust-lang.org/nightly/std/io/trait.Read.html
//! [`BufRead`]: https://doc.rust-lang.org/nightly/std/io/trait.BufRead.html
//! [`Write`]: https://doc.rust-lang.org/nightly/std/io/trait.Write.html

#![deny(unsafe_code)]

mod binding;
mod common;
pub mod lz4;
pub mod lz4_hc;
pub mod lz4_hc_stream;
pub mod lz4_stream;
pub mod lz4f;
pub mod lz4f_stream;

use common::Result;
pub use common::{version_number, version_string, LZ4Error, Report};
