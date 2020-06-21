#![cfg(feature = "use-tokio")]

use super::Decompressor;
use crate::{
    lz4f::{DecompressorBuilder, Error, FrameInfo, Result},
    Buffer,
};
use pin_project::pin_project;
use std::{
    convert::TryInto,
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
#[cfg_attr(docsrs, doc(cfg(feature = "use-tokio")))]
#[pin_project]
pub struct AsyncWriteDecompressor<'a, W: AsyncWrite + Unpin> {
    #[pin]
    device: W,
    inner: Decompressor<'a>,
    consumed: usize,
}

impl<'a, W: AsyncWrite + Unpin> AsyncWriteDecompressor<'a, W> {
    pub fn new(writer: W) -> Result<Self> {
        DecompressorBuilder::new(writer).build()
    }

    fn from_builder(writer: W, capacity: usize) -> Result<Self> {
        Ok(Self {
            device: writer,
            inner: Decompressor::new(capacity)?,
            consumed: 0,
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
        if let Poll::Ready(len) = me.device.poll_write(cx, &me.inner.buf()[..*me.consumed])? {
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
        let report = me.inner.decompress(buf)?;
        let _ = me.write_buffer(cx)?;
        Poll::Ready(Ok(report.src_len().unwrap()))
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<tokio::io::Result<()>> {
        if let Poll::Ready(_) = Pin::new(&mut *self).write_buffer(cx)? {
            self.project().device.poll_flush(cx)
        } else {
            Poll::Pending
        }
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<tokio::io::Result<()>> {
        Pin::new(&mut *self).poll_flush(cx)
    }
}

impl<'a, W: AsyncWrite + Unpin> TryInto<AsyncWriteDecompressor<'a, W>> for DecompressorBuilder<W> {
    type Error = Error;
    fn try_into(self) -> Result<AsyncWriteDecompressor<'a, W>> {
        AsyncWriteDecompressor::from_builder(self.device, self.capacity)
    }
}
