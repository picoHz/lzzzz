#![cfg(feature = "tokio-io")]

use super::{Compressor, CompressorBuilder, Dictionary, Preferences};
use pin_project::pin_project;
use std::{
    convert::TryInto,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncBufRead, AsyncRead, Result};

#[cfg_attr(docsrs, doc(cfg(feature = "tokio-io")))]
#[pin_project]
pub struct AsyncBufReadCompressor<B: AsyncBufRead + Unpin> {
    #[pin]
    device: B,
    inner: Compressor,
    consumed: usize,
}

impl<B: AsyncBufRead + Unpin> AsyncBufReadCompressor<B> {
    pub(crate) fn new(bufreader: B, pref: Preferences, dict: Option<Dictionary>) -> crate::Result<Self> {
        Ok(Self {
            device: bufreader,
            inner: Compressor::new(pref, dict)?,
            consumed: 0,
        })
    }
}

impl<B: AsyncBufRead + Unpin> AsyncRead for AsyncBufReadCompressor<B> {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context, buf: &mut [u8]) -> Poll<Result<usize>> {
        let mut me = self.project();
        let inner_buf = match me.device.as_mut().poll_fill_buf(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(r) => r,
        }?;
        let consumed = if inner_buf.is_empty() {
            me.inner.end(false)?;
            if me.inner.buf().is_empty() {
                return Poll::Ready(Ok(0));
            }
            0
        } else {
            me.inner.update(inner_buf, false)?;
            inner_buf.len()
        };
        me.device.consume(consumed);
        let len = std::cmp::min(buf.len(), me.inner.buf().len() - *me.consumed);
        buf[..len].copy_from_slice(&me.inner.buf()[*me.consumed..][..len]);
        *me.consumed += len;
        if *me.consumed >= me.inner.buf().len() {
            me.inner.clear_buf();
            *me.consumed = 0;
        }
        Poll::Ready(Ok(len))
    }
}

impl<B: AsyncBufRead + Unpin> AsyncBufRead for AsyncBufReadCompressor<B> {
    fn poll_fill_buf(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<&[u8]>> {
        match Pin::new(&mut *self).poll_read(cx, &mut []) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(r) => r?,
        };
        let me = self.project();
        Poll::Ready(Ok(&me.inner.buf()[*me.consumed..]))
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        let me = self.project();
        *me.consumed += amt;
        if *me.consumed >= me.inner.buf().len() {
            me.inner.clear_buf();
            *me.consumed = 0;
        }
    }
}

impl<B: AsyncBufRead + Unpin> TryInto<AsyncBufReadCompressor<B>> for CompressorBuilder<B> {
    type Error = crate::Error;
    fn try_into(self) -> crate::Result<AsyncBufReadCompressor<B>> {
        AsyncBufReadCompressor::new(self.device, self.pref, self.dict)
    }
}

#[cfg(test)]
mod tests {
    use crate::lz4f::{compressor::AsyncBufReadCompressor, CompressorBuilder};
    use tokio::{fs::File, io::BufReader, prelude::*, runtime::Runtime};

    #[tokio::test]
    async fn async_read() -> std::io::Result<()> {
        let mut file = BufReader::new(File::open("foo").await?);
        let mut file = CompressorBuilder::new(&mut file).build::<AsyncBufReadCompressor<_>>()?;
        let mut contents = vec![];
        file.read_to_end(&mut contents).await?;
        Ok(())
    }
}
