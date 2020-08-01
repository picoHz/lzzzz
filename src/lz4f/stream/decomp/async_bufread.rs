#![cfg(feature = "async-io")]

use super::Decompressor;
use crate::{
    common::DEFAULT_BUF_SIZE,
    lz4f::{FrameInfo, Result},
};
use futures_lite::{AsyncBufRead, AsyncRead, AsyncReadExt};
use pin_project::pin_project;
use std::{
    borrow::Cow,
    fmt, io,
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
/// use async_std::{fs::File, io::BufReader, prelude::*};
/// use lzzzz::lz4f::AsyncBufReadDecompressor;
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
    pub(super) inner: R,
    decomp: Decompressor<'a>,
    buf: Vec<u8>,
    inner_consumed: usize,
    consumed: usize,
}

impl<'a, R: AsyncBufRead + Unpin> AsyncBufReadDecompressor<'a, R> {
    /// Creates a new `AsyncBufReadDecompressor<R>`.
    pub fn new(reader: R) -> Result<Self> {
        Ok(Self {
            inner: reader,
            decomp: Decompressor::new()?,
            buf: Vec::with_capacity(DEFAULT_BUF_SIZE),
            inner_consumed: 0,
            consumed: 0,
        })
    }

    /// Sets the dictionary.
    pub fn set_dict<D>(&mut self, dict: D)
    where
        D: Into<Cow<'a, [u8]>>,
    {
        self.decomp.set_dict(dict);
    }

    /// Reads the frame header asynchronously and returns `FrameInfo`.
    ///
    /// Calling this function before any `AsyncRead` or `AsyncBufRead` operations
    /// does not consume the frame body.
    pub async fn read_frame_info(&mut self) -> io::Result<FrameInfo> {
        loop {
            if let Some(frame) = self.decomp.frame_info() {
                return Ok(frame);
            }
            self.decomp.decode_header_only(true);
            let _ = self.read(&mut []).await?;
            self.decomp.decode_header_only(false);
        }
    }

    /// Returns a mutable reference to the reader.
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    /// Returns a shared reference to the reader.
    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    /// Returns ownership of the reader.
    pub fn into_inner(self) -> R {
        self.inner
    }

    fn fill_buf(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        let mut me = self.project();

        let orig_len = me.buf.len();
        #[allow(unsafe_code)]
        unsafe {
            me.buf.set_len(me.buf.capacity());
        }

        let len = me.inner.as_mut().poll_read(cx, &mut me.buf[orig_len..]);
        me.buf.resize(
            orig_len
                + match len {
                    Poll::Ready(Ok(len)) => len,
                    _ => 0,
                },
            0,
        );

        match len {
            Poll::Pending => {
                if me.decomp.buf().is_empty() {
                    return Poll::Pending;
                } else {
                    Ok(0)
                }
            }
            Poll::Ready(r) => r,
        }?;

        *me.inner_consumed += me.decomp.decompress(&me.buf[*me.inner_consumed..])?;
        if *me.inner_consumed >= me.buf.len() {
            *me.inner_consumed = 0;
            me.buf.clear();
        }

        if me.decomp.frame_info().is_none() {
            Poll::Pending
        } else {
            Poll::Ready(Ok(()))
        }
    }
}

impl<R> fmt::Debug for AsyncBufReadDecompressor<'_, R>
where
    R: AsyncBufRead + Unpin + fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("AsyncBufReadDecompressor")
            .field("reader", &self.inner)
            .finish()
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
            let len = std::cmp::min(buf.len(), me.decomp.buf().len() - *me.consumed);
            buf[..len].copy_from_slice(&me.decomp.buf()[*me.consumed..][..len]);
            *me.consumed += len;
            if *me.consumed >= me.decomp.buf().len() {
                me.decomp.clear_buf();
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
            Poll::Ready(Ok(&me.decomp.buf()[*me.consumed..]))
        }
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        let me = self.project();
        *me.consumed += amt;
        if *me.consumed >= me.decomp.buf().len() {
            me.decomp.clear_buf();
            *me.consumed = 0;
        }
    }
}
