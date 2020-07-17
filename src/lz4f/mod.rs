//! LZ4 frame format.
//!
//! # Async streaming
//!
//! The `tokio-io` feature flag enables asynchronous streaming compressors and decompressors.
//!
//! ```toml
//! lzzzz = { version = "...", features = ["tokio-io"] }
//! ```

mod api;
mod binding;
mod dictionary;
mod error;
mod frame;
mod frame_info;
mod preferences;
mod stream;

pub use dictionary::*;
pub use error::*;
pub use frame::*;
pub use frame_info::*;
pub use preferences::*;
pub use stream::{comp::*, decomp::*};
