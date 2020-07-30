#![cfg(feature = "async-io")]

use super::{Compressor, Dictionary, Preferences};
use crate::lz4f::Result;
use futures_lite::AsyncWrite;
use pin_project::pin_project;
use std::{
    fmt, io,
    marker::Unpin,
    pin::Pin,
    task::{Context, Poll},
};

/// The [`AsyncWrite`]-based streaming compressor.
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
/// use async_std::fs::File;
/// use futures_lite::*;
/// use lzzzz::lz4f::AsyncWriteCompressor;
///
/// let mut f = File::create("foo.lz4").await?;
/// let mut w = AsyncWriteCompressor::new(&mut f, Default::default())?;
///
/// w.write_all(b"Hello world!").await?;
///
/// // You have to call close() to finalize the frame.
/// w.close().await?;
/// # Ok::<(), std::io::Error>(())
/// # }).unwrap();
/// # tmp_dir.close().unwrap();
/// ```
///
/// [`AsyncWrite`]: https://docs.rs/futures-io/0.3.5/futures_io/trait.AsyncWrite.html

#[cfg_attr(docsrs, doc(cfg(feature = "async-io")))]
#[pin_project]
pub struct AsyncWriteCompressor<W: AsyncWrite + Unpin> {
    #[pin]
    device: W,
    inner: Compressor,
    consumed: usize,
    state: State,
}

enum State {
    None,
    Write,
    Flush,
    Shutdown,
}

impl<W: AsyncWrite + Unpin> AsyncWriteCompressor<W> {
    /// Creates a new `AsyncWriteCompressor<W>`.
    pub fn new(writer: W, prefs: Preferences) -> Result<Self> {
        Ok(Self {
            device: writer,
            inner: Compressor::new(prefs, None)?,
            consumed: 0,
            state: State::None,
        })
    }

    /// Creates a new `AsyncWriteCompressor<W>` with a dictionary.
    pub fn with_dict(writer: W, prefs: Preferences, dict: Dictionary) -> Result<Self> {
        Ok(Self {
            device: writer,
            inner: Compressor::new(prefs, Some(dict))?,
            consumed: 0,
            state: State::None,
        })
    }

    fn write_buffer(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let me = self.project();
        if let Poll::Ready(len) = me.device.poll_write(cx, &me.inner.buf()[*me.consumed..])? {
            *me.consumed += len;
            if *me.consumed >= me.inner.buf().len() {
                *me.consumed = 0;
                me.inner.clear_buf();
            }
        }
        if me.inner.buf().is_empty() {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }
}

impl<W> fmt::Debug for AsyncWriteCompressor<W>
where
    W: AsyncWrite + Unpin + fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("AsyncWriteCompressor")
            .field("writer", &self.device)
            .field("prefs", &self.inner.prefs())
            .finish()
    }
}

impl<W: AsyncWrite + Unpin> AsyncWrite for AsyncWriteCompressor<W> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let mut me = Pin::new(&mut *self);
        if let State::None = me.state {
            me.inner.update(buf, false)?;
            me.state = State::Write;
        } else if let State::Shutdown = me.state {
            return Poll::Ready(Ok(0));
        }
        if let State::Write = me.state {
            if let Poll::Ready(_) = me.as_mut().write_buffer(cx)? {
                me.state = State::None;
                return Poll::Ready(Ok(buf.len()));
            }
        }
        Poll::Pending
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        let mut me = Pin::new(&mut *self);
        if let State::None = me.state {
            me.inner.flush(false)?;
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

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        let mut me = Pin::new(&mut *self);
        if let State::None = me.state {
            me.inner.end(false)?;
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
