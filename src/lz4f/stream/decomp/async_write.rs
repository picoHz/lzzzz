#![cfg(feature = "async-io")]

use super::Decompressor;
use crate::lz4f::{FrameInfo, Result};
use futures_lite::AsyncWrite;
use pin_project::pin_project;
use std::{
    borrow::Cow,
    fmt, io,
    marker::Unpin,
    pin::Pin,
    task::{Context, Poll},
};

/// The [`AsyncWrite`]-based streaming decompressor.
///
/// # Example
///
/// ```
/// # use std::env;
/// # use std::path::Path;
/// # use lzzzz::{Error, Result};
/// # let tmp_dir = assert_fs::TempDir::new().unwrap().into_persistent();
/// # env::set_current_dir(tmp_dir.path()).unwrap();
/// # smol::run(async {
/// use async_std::{fs::File, prelude::*};
/// use lzzzz::lz4f::{compress_to_vec, AsyncWriteDecompressor};
///
/// let mut f = File::create("foo.txt").await?;
/// let mut w = AsyncWriteDecompressor::new(&mut f)?;
///
/// let mut buf = Vec::new();
/// compress_to_vec(b"Hello world!", &mut buf, &Default::default())?;
///
/// w.write_all(&buf).await?;
/// # Ok::<(), std::io::Error>(())
/// # }).unwrap();
/// # tmp_dir.close().unwrap();
/// ```
///
/// [`AsyncWrite`]: https://docs.rs/futures-io/0.3.5/futures_io/trait.AsyncWrite.html

#[cfg_attr(docsrs, doc(cfg(feature = "async-io")))]
#[pin_project]
pub struct AsyncWriteDecompressor<'a, W: AsyncWrite + Unpin> {
    #[pin]
    inner: W,
    decomp: Decompressor<'a>,
    consumed: usize,
    state: State,
}

enum State {
    None,
    Write(usize),
    Flush,
    Shutdown,
}

impl<'a, W: AsyncWrite + Unpin> AsyncWriteDecompressor<'a, W> {
    /// Creates a new `AsyncWriteDecompressor<W>`.
    pub fn new(writer: W) -> Result<Self> {
        Ok(Self {
            inner: writer,
            decomp: Decompressor::new()?,
            consumed: 0,
            state: State::None,
        })
    }

    /// Sets the dictionary.
    pub fn set_dict<D>(&mut self, dict: D)
    where
        D: Into<Cow<'a, [u8]>>,
    {
        self.decomp.set_dict(dict);
    }

    /// Returns `FrameInfo` if the frame header is already decoded.
    /// Otherwise, returns `None`.
    pub fn frame_info(&self) -> Option<FrameInfo> {
        self.decomp.frame_info()
    }

    /// Sets the 'header-only' mode.
    ///
    /// When the 'header-only' mode is enabled, the decompressor doesn't
    /// consume the frame body and `poll_write()` always returns `Ok(0)`
    /// if the frame header is already decoded.
    pub fn decode_header_only(&mut self, flag: bool) {
        self.decomp.decode_header_only(flag);
    }

    fn write_buffer(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let me = self.project();
        if let Poll::Ready(len) = me.inner.poll_write(cx, &me.decomp.buf()[*me.consumed..])? {
            *me.consumed += len;
            if *me.consumed >= me.decomp.buf().len() {
                *me.consumed = 0;
                me.decomp.clear_buf();
            }
        }
        if me.decomp.buf().is_empty() {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }
}

impl<W> fmt::Debug for AsyncWriteDecompressor<'_, W>
where
    W: AsyncWrite + Unpin + fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("AsyncWriteDecompressor")
            .field("writer", &self.inner)
            .finish()
    }
}

impl<W: AsyncWrite + Unpin> AsyncWrite for AsyncWriteDecompressor<'_, W> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let mut me = Pin::new(&mut *self);
        if let State::None = me.state {
            me.state = State::Write(me.decomp.decompress(buf)?);
        } else if let State::Shutdown = me.state {
            return Poll::Ready(Ok(0));
        }
        if let State::Write(len) = me.state {
            if let Poll::Ready(_) = me.as_mut().write_buffer(cx)? {
                me.state = State::None;
                return Poll::Ready(Ok(len));
            }
        }
        Poll::Pending
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let mut me = Pin::new(&mut *self);
        if let State::None = me.state {
            me.state = State::Flush;
        } else if let State::Shutdown = me.state {
            return Poll::Ready(Ok(()));
        }
        if let State::Flush = me.state {
            if let Poll::Ready(_) = me.as_mut().write_buffer(cx)? {
                me.state = State::None;
                return Poll::Ready(Ok(()));
            }
        }
        Poll::Pending
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let mut me = Pin::new(&mut *self);
        if let State::None = me.state {
            me.state = State::Shutdown;
        } else if let State::Shutdown = me.state {
            return Poll::Ready(Ok(()));
        }
        if let State::Shutdown = me.state {
            if let Poll::Ready(_) = me.as_mut().write_buffer(cx)? {
                return Poll::Ready(Ok(()));
            }
        }
        Poll::Pending
    }
}
