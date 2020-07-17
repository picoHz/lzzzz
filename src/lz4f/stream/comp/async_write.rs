#![cfg(feature = "tokio-io")]

use super::{Compressor, Dictionary, Preferences};
use crate::lz4f::Result;
use pin_project::pin_project;
use std::{
    marker::Unpin,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::AsyncWrite;

/// The [`AsyncWrite`]-based streaming compressor.
///
/// # Examples
///
/// ```
/// # use std::env;
/// # use std::path::Path;
/// # use lzzzz::{Error, Result};
/// # let tmp_dir = assert_fs::TempDir::new().unwrap().into_persistent();
/// # env::set_current_dir(tmp_dir.path()).unwrap();
/// # let mut rt = tokio::runtime::Runtime::new().unwrap();
/// # rt.block_on(async {
/// use lzzzz::lz4f::comp::AsyncWriteCompressor;
/// use tokio::{fs::File, prelude::*};
///
/// let mut f = File::create("foo.lz4").await?;
/// let mut w = AsyncWriteCompressor::new(&mut f, Default::default())?;
///
/// w.write_all(b"hello, world!").await?;
///
/// // You have to call shutdown() to finalize the frame.
/// w.shutdown().await?;
/// # Ok::<(), tokio::io::Error>(())
/// # }).unwrap();
/// # tmp_dir.close().unwrap();
/// ```
///
/// [`AsyncWrite`]: https://docs.rs/tokio/latest/tokio/io/trait.AsyncWrite.html
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-io")))]
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
    pub fn new(writer: W, prefs: Preferences) -> Result<Self> {
        Ok(Self {
            device: writer,
            inner: Compressor::new(prefs, None)?,
            consumed: 0,
            state: State::None,
        })
    }

    pub fn with_dict(writer: W, prefs: Preferences, dict: Dictionary) -> Result<Self> {
        Ok(Self {
            device: writer,
            inner: Compressor::new(prefs, Some(dict))?,
            consumed: 0,
            state: State::None,
        })
    }

    fn write_buffer(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<tokio::io::Result<()>> {
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

impl<W: AsyncWrite + Unpin> AsyncWrite for AsyncWriteCompressor<W> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<tokio::io::Result<usize>> {
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

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<tokio::io::Result<()>> {
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

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<tokio::io::Result<()>> {
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
