#![cfg(feature = "tokio-io")]

use super::{Compressor, CompressorBuilder, Dictionary, Preferences, State, LZ4F_HEADER_SIZE_MAX};
use futures::{future::FutureExt, ready};
use pin_project::{pin_project, project};
use pin_utils::pin_mut;
use std::{
    convert::TryInto,
    marker::Unpin,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncWrite, AsyncWriteExt, Result};

#[cfg_attr(docsrs, doc(cfg(feature = "tokio-io")))]
#[pin_project]
pub struct AsyncWriteCompressor<W: AsyncWrite + Unpin> {
    #[pin]
    device: W,
    inner: Compressor,
}

impl<W: AsyncWrite + Unpin> AsyncWriteCompressor<W> {
    fn new(writer: W, pref: Preferences, dict: Option<Dictionary>) -> crate::Result<Self> {
        Ok(Self {
            device: writer,
            inner: Compressor::new(pref, dict)?,
        })
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.inner.update(buf, false)?;
        self.device.write_all(self.inner.buf()).await?;
        self.inner.clear_buf();
        Ok(buf.len())
    }

    async fn flush(&mut self) -> Result<()> {
        self.inner.flush(false)?;
        self.device.write_all(self.inner.buf()).await?;
        self.device.flush().await
    }

    async fn shutdown(&mut self) -> Result<()> {
        self.inner.end(false)?;
        self.device.write_all(self.inner.buf()).await?;
        self.inner.clear_buf();
        self.device.flush().await
    }
}

impl<W: AsyncWrite + Unpin> AsyncWrite for AsyncWriteCompressor<W> {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context, buf: &[u8]) -> Poll<Result<usize>> {
        let mut me = Pin::new(&mut *self);
        let future = me.write(buf);
        pin_mut!(future);
        future.poll_unpin(cx)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<()>> {
        let mut me = Pin::new(&mut *self);
        let future = me.flush();
        pin_mut!(future);
        future.poll_unpin(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<()>> {
        let mut me = Pin::new(&mut *self);
        let future = me.shutdown();
        pin_mut!(future);
        future.poll_unpin(cx)
    }
}

impl<W: AsyncWrite + Unpin> TryInto<AsyncWriteCompressor<W>> for CompressorBuilder<W> {
    type Error = crate::Error;
    fn try_into(self) -> crate::Result<AsyncWriteCompressor<W>> {
        AsyncWriteCompressor::new(self.device, self.pref, self.dict)
    }
}
