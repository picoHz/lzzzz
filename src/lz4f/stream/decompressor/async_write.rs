#![cfg(feature = "tokio-io")]

use tokio::io::AsyncWrite;

#[cfg_attr(docsrs, doc(cfg(feature = "tokio-io")))]
pub struct AsyncWriteDecompressor {}
