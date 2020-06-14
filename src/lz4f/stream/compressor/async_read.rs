#![cfg(feature = "tokio-io")]

use tokio::io::AsyncRead;

#[cfg_attr(docsrs, doc(cfg(feature = "tokio-io")))]
pub struct AsyncReadCompressor {}
