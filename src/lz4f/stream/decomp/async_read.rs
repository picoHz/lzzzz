#![cfg(feature = "async-io")]

use super::AsyncBufReadDecompressor;
use crate::lz4f::{FrameInfo, Result};
use async_std::io::BufReader;
use futures_lite::AsyncRead;
use pin_project::pin_project;
use std::{
    borrow::Cow,
    fmt, io,
    pin::Pin,
    task::{Context, Poll},
};

/// The [`AsyncRead`]-based streaming decompressor.
///
/// # Example
///
/// ```
/// # use std::env;
/// # use std::path::Path;
/// # use lzzzz::{Error, Result};
/// # use assert_fs::prelude::*;
/// # let tmp_dir = assert_fs::TempDir::new().unwrap().into_persistent();
/// # env::set_current_dir(tmp_dir.path()).unwrap();
/// #
/// # let mut buf = Vec::new();
/// # lzzzz::lz4f::compress_to_vec(b"Hello world!", &mut buf, &Default::default()).unwrap();
/// # tmp_dir.child("foo.lz4").write_binary(&buf).unwrap();
/// #
/// # smol::run(async {
/// use async_std::{fs::File, prelude::*};
/// use lzzzz::lz4f::AsyncReadDecompressor;
///
/// let mut f = File::open("foo.lz4").await?;
/// let mut r = AsyncReadDecompressor::new(&mut f)?;
///
/// let mut buf = Vec::new();
/// r.read_to_end(&mut buf).await?;
/// # Ok::<(), std::io::Error>(())
/// # }).unwrap();
/// # tmp_dir.close().unwrap();
/// ```
///
/// [`AsyncRead`]: https://docs.rs/futures-io/0.3.5/futures_io/trait.AsyncRead.html

#[cfg_attr(docsrs, doc(cfg(feature = "async-io")))]
#[pin_project]
pub struct AsyncReadDecompressor<'a, R: AsyncRead + Unpin> {
    #[pin]
    inner: AsyncBufReadDecompressor<'a, BufReader<R>>,
}

impl<'a, R: AsyncRead + Unpin> AsyncReadDecompressor<'a, R> {
    /// Creates a new `AsyncReadDecompressor<R>`.
    pub fn new(reader: R) -> Result<Self> {
        Ok(Self {
            inner: AsyncBufReadDecompressor::new(BufReader::new(reader))?,
        })
    }

    /// Sets the dictionary.
    pub fn set_dict<D>(&mut self, dict: D)
    where
        D: Into<Cow<'a, [u8]>>,
    {
        self.inner.set_dict(dict);
    }

    /// Reads the frame header asynchronously and returns `FrameInfo`.
    ///
    /// Calling this function before any `AsyncRead` operations
    /// does not consume the frame body.
    pub async fn read_frame_info(&mut self) -> io::Result<FrameInfo> {
        self.inner.read_frame_info().await
    }

    /// Returns a mutable reference to the reader.
    pub fn get_mut(&mut self) -> &mut R {
        self.inner.get_mut().get_mut()
    }

    /// Returns a shared reference to the reader.
    pub fn get_ref(&mut self) -> &R {
        self.inner.get_ref().get_ref()
    }

    /// Returns ownership of the reader.
    pub fn into_inner(self) -> R {
        self.inner.into_inner().into_inner()
    }
}

impl<R> fmt::Debug for AsyncReadDecompressor<'_, R>
where
    R: AsyncRead + Unpin + fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("AsyncReadDecompressor")
            .field("reader", &self.inner.inner.get_ref())
            .finish()
    }
}

impl<'a, R: AsyncRead + Unpin> AsyncRead for AsyncReadDecompressor<'a, R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        self.project().inner.poll_read(cx, buf)
    }
}
