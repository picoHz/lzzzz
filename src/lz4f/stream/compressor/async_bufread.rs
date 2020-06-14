#![cfg(feature = "tokio-io")]

use tokio::io::AsyncBufRead;

#[cfg_attr(docsrs, doc(cfg(feature = "tokio-io")))]
pub struct AsyncBufReadDecompressor {}
