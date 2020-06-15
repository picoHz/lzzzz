#![cfg(feature = "tokio-io")]

use super::{Compressor, CompressorBuilder, Dictionary, Preferences, State, LZ4F_HEADER_SIZE_MAX};
use futures::{future::FutureExt, ready};
use pin_project::{pin_project, project};
use std::{
    convert::TryInto,
    marker::Unpin,
    mem::MaybeUninit,
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

impl<W: AsyncWrite + Unpin> AsyncWrite for AsyncWriteCompressor<W> {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context, buf: &[u8]) -> Poll<Result<usize>> {
        self.project().device.write_all(&[]).poll_unpin(cx);
        Poll::Ready(Ok(0))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<()>> {
        let mut me = self.project();
        if !me.inner.buf().is_empty() {
            ready!(me.device.as_mut().write_all(me.inner.buf()).poll_unpin(cx))?;
            me.inner.clear_buf();
            return Poll::Pending;
        }
        me.inner.end(false)?;
        Poll::Ready(Ok(()))
    }
}
