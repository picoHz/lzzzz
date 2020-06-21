#![cfg(feature = "use-tokio")]

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

/// AsyncWrite-based streaming compressor
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
/// let mut w = AsyncWriteCompressor::new(&mut f)?;
///
/// w.write_all(b"hello, world!").await?;
/// # Ok::<(), tokio::io::Error>(())
/// # }).unwrap();
/// # tmp_dir.close().unwrap();
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "use-tokio")))]
#[pin_project]
pub struct AsyncWriteCompressor<W: AsyncWrite + Unpin> {
    #[pin]
    device: W,
    inner: Compressor,
    consumed: usize,
}

impl<W: AsyncWrite + Unpin> AsyncWriteCompressor<W> {
    pub fn new(writer: W) -> crate::lz4f::Result<Self> {
        CompressorBuilder::new(writer).build()
    }

    fn from_builder(
        writer: W,
        pref: Preferences,
        dict: Option<Dictionary>,
    ) -> crate::lz4f::Result<Self> {
        Ok(Self {
            device: writer,
            inner: Compressor::new(pref, dict)?,
            consumed: 0,
        })
    }

    fn write_buffer(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
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

impl<W: AsyncWrite + Unpin> AsyncWrite for AsyncWriteCompressor<W> {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context, buf: &[u8]) -> Poll<Result<usize>> {
        let mut me = Pin::new(&mut *self);
        me.inner.update(buf, false)?;
        let _ = me.write_buffer(cx)?;
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<()>> {
        let mut me = Pin::new(&mut *self);
        me.inner.flush(false)?;
        me.write_buffer(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<()>> {
        let mut me = Pin::new(&mut *self);
        me.inner.end(false)?;
        me.write_buffer(cx)
    }
}

impl<W: AsyncWrite + Unpin> TryInto<AsyncWriteCompressor<W>> for CompressorBuilder<W> {
    type Error = crate::lz4f::Error;
    fn try_into(self) -> crate::lz4f::Result<AsyncWriteCompressor<W>> {
        AsyncWriteCompressor::from_builder(self.device, self.pref, self.dict)
    }
}

#[cfg(test)]
mod tests {
    use crate::lz4f::{comp::AsyncWriteCompressor, CompressorBuilder};
    use tokio::{fs::File, prelude::*};

    #[tokio::test]
    async fn async_write() -> std::io::Result<()> {
        let mut file = File::create("foo").await?;
        let mut file = CompressorBuilder::new(&mut file).build::<AsyncWriteCompressor<_>>()?;
        file.write_all(b"hello, world!").await
    }
}
