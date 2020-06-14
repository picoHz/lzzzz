#![cfg(feature = "tokio-io")]

use super::{Compressor, CompressorBuilder, Dictionary, Preferences, State, LZ4F_HEADER_SIZE_MAX};
use futures::future::FutureExt;
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
pub struct AsyncWriteCompressor<W: AsyncWrite> {
    #[pin]
    device: W,
    buffer: Vec<u8>,
}

impl<W: AsyncWrite> AsyncWriteCompressor<W> {
    fn aaaa(self: Pin<&mut Self>, buf: &[u8]) -> Poll<Result<usize>> {
        Poll::Ready(Ok(0))
    }
}

impl<W: AsyncWrite> AsyncWrite for AsyncWriteCompressor<W> {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context, buf: &[u8]) -> Poll<Result<usize>> {
        self.as_mut().project().buffer.reserve(1);
        self.project().device.write_all(&[]);
        Poll::Ready(Ok(0))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }
}
