#![cfg(feature = "tokio-io")]

use super::Decompressor;
use crate::{
    lz4f::{FrameInfo, Result},
    Buffer,
};
use pin_project::pin_project;
use std::{
    marker::Unpin,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::AsyncWrite;

/// AsyncWrite-based streaming decompressor
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
/// use lzzzz::lz4f::{compress_to_vec, decomp::AsyncWriteDecompressor};
/// use tokio::{fs::File, prelude::*};
///
/// let mut f = File::create("foo.txt").await?;
/// let mut w = AsyncWriteDecompressor::new(&mut f)?;
///
/// let mut buf = Vec::new();
/// compress_to_vec(b"Hello world!", &mut buf, &Default::default())?;
///
/// w.write_all(&buf).await?;
/// # Ok::<(), tokio::io::Error>(())
/// # }).unwrap();
/// # tmp_dir.close().unwrap();
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-io")))]
#[pin_project]
pub struct AsyncWriteDecompressor<'a, W: AsyncWrite + Unpin> {
    #[pin]
    device: W,
    inner: Decompressor<'a>,
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
    pub fn new(writer: W) -> Result<Self> {
        Ok(Self {
            device: writer,
            inner: Decompressor::new()?,
            consumed: 0,
            state: State::None,
        })
    }

    pub fn set_dict<B>(&mut self, dict: B)
    where
        B: Into<Buffer<'a>>,
    {
        self.inner.set_dict(dict.into());
    }

    pub fn frame_info(&self) -> Option<FrameInfo> {
        self.inner.frame_info()
    }

    pub fn decode_header_only(&mut self, flag: bool) {
        self.inner.decode_header_only(flag);
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

impl<W: AsyncWrite + Unpin> AsyncWrite for AsyncWriteDecompressor<'_, W> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<tokio::io::Result<usize>> {
        let mut me = Pin::new(&mut *self);
        if let State::None = me.state {
            me.state = State::Write(me.inner.decompress(buf)?);
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

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<tokio::io::Result<()>> {
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

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<tokio::io::Result<()>> {
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
