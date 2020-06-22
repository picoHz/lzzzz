//! Yet another liblz4 binding 😴
//!
//! - **Designed for Rust:** Lzzzz is a high-level wrapper of liblz4
//! provides comprehensible API complies with Rust's manner without losing performance and
//! flexibility. You have no concern about memory management and concurrency
//! safety.
//!
//! - **Various Modes:** [`LZ4`], [`LZ4_HC`], [`LZ4F`], [`LZ4 Streaming`],
//! [`LZ4_HC Streaming`] and [`LZ4F Streaming`] are supported.
//!
//! - **Flexible Streaming:** All the compressor/decompressor streams support [`Read`],
//! [`BufRead`] and [`Write`] operations. With `tokio-support` feature, [`AsyncRead`],
//! [`AsyncBufRead`] and [`AsyncWrite`] are also supported.
//!
//! [`LZ4`]: ./lz4/index.html
//! [`LZ4_HC`]: ./lz4_hc/index.html
//! [`LZ4F`]: ./lz4f/index.html
//! [`LZ4 Streaming`]: ./lz4_stream/index.html
//! [`LZ4_HC Streaming`]: ./lz4_hc_stream/index.html
//! [`LZ4F Streaming`]: ./lz4f_stream/index.html
//! [`Read`]: https://doc.rust-lang.org/std/io/trait.Read.html
//! [`BufRead`]: https://doc.rust-lang.org/std/io/trait.BufRead.html
//! [`Write`]: https://doc.rust-lang.org/std/io/trait.Write.html
//! [`AsyncRead`]: https://docs.rs/tokio/latest/tokio/io/trait.AsyncRead.html
//! [`AsyncBufRead`]: https://docs.rs/tokio/latest/tokio/io/trait.AsyncBufRead.html
//! [`AsyncWrite`]: https://docs.rs/tokio/latest/tokio/io/trait.AsyncWrite.html

#![deny(unsafe_code)]
#![deny(clippy::all)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod common;

pub mod lz4;
pub mod lz4_hc;
pub mod lz4f;

pub use common::*;
