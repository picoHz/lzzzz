#![cfg(feature = "tokio-io")]

use super::Decompressor;
use crate::lz4f::{DecompressorBuilder, FrameInfo};
use pin_project::pin_project;
use std::{
    borrow::Cow,
    convert::TryInto,
    marker::Unpin,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncWrite, Result};

/// AsyncWrite-based streaming decompressor
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-io")))]
#[pin_project]
pub struct AsyncWriteDecompressor<'a, W: AsyncWrite + Unpin> {
    #[pin]
    device: W,
    inner: Decompressor<'a>,
    len: usize,
}

impl<'a, W: AsyncWrite + Unpin> AsyncWriteDecompressor<'a, W> {
    fn new(writer: W) -> crate::Result<Self> {
        Ok(Self {
            device: writer,
            inner: Decompressor::new()?,
            len: 0,
        })
    }

    pub fn set_dict(&mut self, dict: Cow<'a, [u8]>) {
        self.inner.set_dict(dict);
    }

    pub fn frame_info(&self) -> Option<FrameInfo> {
        self.inner.get_frame_info().ok()
    }
}

impl<W: AsyncWrite + Unpin> AsyncWrite for AsyncWriteDecompressor<'_, W> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize>> {
        let mut me = Pin::new(&mut *self);
        let report = { me.inner.decompress(buf)? };
        let _ = me.poll_flush(cx)?;
        Poll::Ready(Ok(report.src_len().unwrap()))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        let mut me = self.project();
        if let Poll::Ready(len) = me
            .device
            .as_mut()
            .poll_write(cx, &me.inner.buf()[..*me.len])?
        {
            *me.len += len;
            if *me.len >= me.inner.buf().len() {
                *me.len = 0;
                me.inner.clear_buf();
            }
        }
        me.device.poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut *self).poll_flush(cx)
    }
}

impl<'a, W: AsyncWrite + Unpin> TryInto<AsyncWriteDecompressor<'a, W>> for DecompressorBuilder<W> {
    type Error = crate::Error;
    fn try_into(self) -> crate::Result<AsyncWriteDecompressor<'a, W>> {
        AsyncWriteDecompressor::new(self.device)
    }
}
