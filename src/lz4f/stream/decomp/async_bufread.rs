#![cfg(feature = "async-io")]

use super::Decompressor;
use crate::lz4f::{FrameInfo, Result};
use futures_lite::{AsyncBufRead, AsyncRead, AsyncReadExt};
use pin_project::pin_project;
use std::{
    borrow::Cow,
    io,
    pin::Pin,
    task::{Context, Poll},
};

/// The [`AsyncBufRead`]-based streaming decompressor.
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
/// use lzzzz::lz4f::AsyncBufReadDecompressor;
/// use async_std::{fs::File, io::BufReader, prelude::*};
///
/// let mut f = File::open("foo.lz4").await?;
/// let mut b = BufReader::new(f);
/// let mut r = AsyncBufReadDecompressor::new(&mut b)?;
///
/// let mut buf = Vec::new();
/// r.read_frame_info().await?;
/// r.read_to_end(&mut buf).await?;
/// # Ok::<(), std::io::Error>(())
/// # }).unwrap();
/// # tmp_dir.close().unwrap();
/// ```
///
/// [`AsyncBufRead`]: https://docs.rs/futures-io/0.3.5/futures_io/trait.AsyncBufRead.html

#[cfg_attr(docsrs, doc(cfg(feature = "async-io")))]
#[pin_project]
pub struct AsyncBufReadDecompressor<'a, R: AsyncBufRead + Unpin> {
    #[pin]
    device: R,
    inner: Decompressor<'a>,
    consumed: usize,
}

impl<'a, R: AsyncBufRead + Unpin> AsyncBufReadDecompressor<'a, R> {
    /// Creates a new `AsyncBufReadDecompressor<R>`.
    pub fn new(reader: R) -> Result<Self> {
        Ok(Self {
            device: reader,
            inner: Decompressor::new()?,
            consumed: 0,
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
    /// Calling this function before any `AsyncRead` or `AsyncBufRead` operations
    /// does not consume the frame body.
    pub async fn read_frame_info(&mut self) -> io::Result<FrameInfo> {
        loop {
            if let Some(frame) = self.inner.frame_info() {
                return Ok(frame);
            }
            self.inner.decode_header_only(true);
            let _ = self.read(&mut []).await?;
            self.inner.decode_header_only(false);
        }
    }

    fn fill_buf(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        let mut me = self.project();
        let inner_buf = match me.device.as_mut().poll_fill_buf(cx) {
            Poll::Pending => {
                if me.inner.buf().is_empty() {
                    return Poll::Pending;
                } else {
                    Ok(&[][..])
                }
            }
            Poll::Ready(r) => r,
        }?;
        let consumed = me.inner.decompress(inner_buf)?;
        me.device.as_mut().consume(consumed);
        Poll::Ready(Ok(()))
    }
}

impl<'a, R: AsyncBufRead + Unpin> AsyncRead for AsyncBufReadDecompressor<'a, R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        if let Poll::Pending = Pin::new(&mut *self).fill_buf(cx)? {
            Poll::Pending
        } else {
            let me = self.project();
            let len = std::cmp::min(buf.len(), me.inner.buf().len() - *me.consumed);
            buf[..len].copy_from_slice(&me.inner.buf()[*me.consumed..][..len]);
            *me.consumed += len;
            if *me.consumed >= me.inner.buf().len() {
                me.inner.clear_buf();
                *me.consumed = 0;
            }
            Poll::Ready(Ok(len))
        }
    }
}

impl<'a, R: AsyncBufRead + Unpin> AsyncBufRead for AsyncBufReadDecompressor<'a, R> {
    fn poll_fill_buf(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<&[u8]>> {
        if let Poll::Pending = Pin::new(&mut *self).fill_buf(cx)? {
            Poll::Pending
        } else {
            let me = self.project();
            Poll::Ready(Ok(&me.inner.buf()[*me.consumed..]))
        }
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        let me = self.project();
        *me.consumed += amt;
        if *me.consumed >= me.inner.buf().len() {
            me.inner.clear_buf();
            *me.consumed = 0;
        }
    }
}
