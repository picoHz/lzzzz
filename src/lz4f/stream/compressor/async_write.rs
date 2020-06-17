#![cfg(feature = "tokio-io")]

use super::{Compressor, Dictionary, Preferences};
use crate::lz4f::CompressorBuilder;
use pin_project::pin_project;
use std::{
    convert::TryInto,
    marker::Unpin,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncWrite, Result};

enum State {
    None,
    Write,
    Flush,
    Shutdown,
}

/// AsyncWrite-based streaming compressor
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-io")))]
#[pin_project]
pub struct AsyncWriteCompressor<W: AsyncWrite + Unpin> {
    #[pin]
    device: W,
    inner: Compressor,
    state: State,
    len: usize,
}

impl<W: AsyncWrite + Unpin> AsyncWriteCompressor<W> {
    fn new(writer: W, pref: Preferences, dict: Option<Dictionary>) -> crate::Result<Self> {
        Ok(Self {
            device: writer,
            inner: Compressor::new(pref, dict)?,
            state: State::None,
            len: 0,
        })
    }
}

impl<W: AsyncWrite + Unpin> AsyncWrite for AsyncWriteCompressor<W> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context, buf: &[u8]) -> Poll<Result<usize>> {
        let me = self.project();
        if let State::None = me.state {
            me.inner.clear_buf();
            *me.len = 0;
            me.inner.update(buf, false)?;
            *me.state = State::Write;
        }
        if let State::Write = me.state {
            *me.len += match me.device.poll_write(cx, &me.inner.buf()[*me.len..]) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Ok(len)) => len,
                Poll::Ready(Err(err)) => {
                    *me.state = State::None;
                    return Poll::Ready(Err(err));
                }
            };
            if *me.len >= me.inner.buf().len() {
                *me.state = State::None;
                return Poll::Ready(Ok(buf.len()));
            }
        }
        Poll::Pending
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<()>> {
        let me = self.project();
        if let State::None = me.state {
            me.inner.clear_buf();
            *me.len = 0;
            me.inner.flush(false)?;
            *me.state = State::Flush;
        }
        if let State::Flush = me.state {
            *me.len += match me.device.poll_write(cx, &me.inner.buf()[*me.len..]) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Ok(len)) => len,
                Poll::Ready(Err(err)) => {
                    *me.state = State::None;
                    return Poll::Ready(Err(err));
                }
            };
            if *me.len >= me.inner.buf().len() {
                *me.state = State::None;
                return Poll::Ready(Ok(()));
            }
        }
        Poll::Pending
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<()>> {
        let me = self.project();
        if let State::None = me.state {
            me.inner.clear_buf();
            *me.len = 0;
            me.inner.end(false)?;
            *me.state = State::Shutdown;
        }
        if let State::Shutdown = me.state {
            *me.len += match me.device.poll_write(cx, &me.inner.buf()[*me.len..]) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Ok(len)) => len,
                Poll::Ready(Err(err)) => {
                    *me.state = State::None;
                    return Poll::Ready(Err(err));
                }
            };
            if *me.len >= me.inner.buf().len() {
                *me.state = State::None;
                return Poll::Ready(Ok(()));
            }
        }
        Poll::Pending
    }
}

impl<W: AsyncWrite + Unpin> TryInto<AsyncWriteCompressor<W>> for CompressorBuilder<W> {
    type Error = crate::Error;
    fn try_into(self) -> crate::Result<AsyncWriteCompressor<W>> {
        AsyncWriteCompressor::new(self.device, self.pref, self.dict)
    }
}

#[cfg(test)]
mod tests {
    use crate::lz4f::{compressor::AsyncWriteCompressor, CompressorBuilder};
    use tokio::{fs::File, prelude::*, runtime::Runtime};

    #[tokio::test]
    async fn async_write() -> std::io::Result<()> {
        let mut file = File::create("foo").await?;
        let mut file = CompressorBuilder::new(&mut file).build::<AsyncWriteCompressor<_>>()?;
        file.write_all(b"hello, world!").await
    }
}
