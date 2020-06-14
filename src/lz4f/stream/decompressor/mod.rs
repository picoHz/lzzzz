//! Streaming Decompressors
mod async_bufread;
mod async_read;
mod async_write;
mod bufread;
mod read;
mod write;

pub use {bufread::*, read::*, write::*};

#[cfg(feature = "tokio-io")]
pub use {async_bufread::*, async_read::*, async_write::*};