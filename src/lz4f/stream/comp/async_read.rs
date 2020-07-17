#![cfg(feature = "tokio-io")]

use super::{AsyncBufReadCompressor, Dictionary, Preferences};
use crate::lz4f::Result;
use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, BufReader};

/// The [`AsyncRead`]-based streaming compressor.
///
/// # Examples
///
/// ```
/// # use std::env;
/// # use std::path::Path;
/// # use lzzzz::{Error, Result};
/// # use assert_fs::prelude::*;
/// # let tmp_dir = assert_fs::TempDir::new().unwrap().into_persistent();
/// # env::set_current_dir(tmp_dir.path()).unwrap();
/// #
/// # tmp_dir.child("foo.txt").write_str("Hello").unwrap();
/// #
/// # let mut rt = tokio::runtime::Runtime::new().unwrap();
/// # rt.block_on(async {
/// use lzzzz::lz4f::AsyncReadCompressor;
/// use tokio::{fs::File, prelude::*};
///
/// let mut f = File::open("foo.txt").await?;
/// let mut r = AsyncReadCompressor::new(&mut f, Default::default())?;
///
/// let mut buf = Vec::new();
/// r.read_to_end(&mut buf).await?;
/// # Ok::<(), tokio::io::Error>(())
/// # }).unwrap();
/// # tmp_dir.close().unwrap();
/// ```
///
/// [`AsyncRead`]: https://docs.rs/tokio/latest/tokio/io/trait.AsyncRead.html

#[cfg_attr(docsrs, doc(cfg(feature = "tokio-io")))]
#[pin_project]
pub struct AsyncReadCompressor<R: AsyncRead + Unpin> {
    #[pin]
    inner: AsyncBufReadCompressor<BufReader<R>>,
}

impl<R: AsyncRead + Unpin> AsyncReadCompressor<R> {
    /// Creates a new `AsyncBufReadCompressor<R>`.
    pub fn new(reader: R, prefs: Preferences) -> Result<Self> {
        Ok(Self {
            inner: AsyncBufReadCompressor::new(BufReader::new(reader), prefs)?,
        })
    }

    /// Creates a new `AsyncBufReadCompressor<R>` with a dictionary.
    pub fn with_dict(reader: R, prefs: Preferences, dict: Dictionary) -> Result<Self> {
        Ok(Self {
            inner: AsyncBufReadCompressor::with_dict(BufReader::new(reader), prefs, dict)?,
        })
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for AsyncReadCompressor<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<tokio::io::Result<usize>> {
        self.project().inner.poll_read(cx, buf)
    }
}
