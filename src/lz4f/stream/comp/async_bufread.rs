#![cfg(feature = "use-tokio")]

use super::{Compressor, Dictionary, Preferences};
use crate::lz4f::CompressorBuilder;
use pin_project::pin_project;
use std::{
    cmp,
    convert::TryInto,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncBufRead, AsyncRead, Result};

/// AsyncBufRead-based streaming compressor
///
/// # Examples
///
/// ```
/// # use std::env;
/// # use std::path::Path;
/// # use lzzzz::{Error, Result};
/// # use assert_fs::prelude::*;
/// # let tmp_dir = assert_fs::TempDir::new().unwrap().into_persistent();
/// # env::set_current_dir(tmp_dir.path()).unwrap();
/// #
/// # tmp_dir.child("foo.txt").write_str("Hello").unwrap();
/// #
/// # let mut rt = tokio::runtime::Runtime::new().unwrap();
/// # rt.block_on(async {
/// use lzzzz::lz4f::comp::AsyncBufReadCompressor;
/// use tokio::{fs::File, io::BufReader, prelude::*};
///
/// let mut f = File::open("foo.txt").await?;
/// let mut b = BufReader::new(f);
/// let mut r = AsyncBufReadCompressor::new(&mut b)?;
///
/// let mut buf = Vec::new();
/// r.read_to_end(&mut buf).await?;
/// # Ok::<(), tokio::io::Error>(())
/// # }).unwrap();
/// # tmp_dir.close().unwrap();
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "use-tokio")))]
#[pin_project]
pub struct AsyncBufReadCompressor<R: AsyncBufRead + Unpin> {
    #[pin]
    device: R,
    inner: Compressor,
    consumed: usize,
}

impl<R: AsyncBufRead + Unpin> AsyncBufReadCompressor<R> {
    pub fn new(reader: R) -> crate::Result<Self> {
        Self::from_builder(reader, Default::default(), None)
    }

    pub(crate) fn from_builder(
        reader: R,
        pref: Preferences,
        dict: Option<Dictionary>,
    ) -> crate::Result<Self> {
        Ok(Self {
            device: reader,
            inner: Compressor::new(pref, dict)?,
            consumed: 0,
        })
    }

    fn fill_buf(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<()>> {
        let mut me = self.project();
        let inner_buf = match me.device.as_mut().poll_fill_buf(cx) {
            Poll::Pending => {
                if me.inner.buf().is_empty() {
                    return Poll::Pending;
                } else {
                    Ok(&[][..])
                }
            }
            Poll::Ready(r) => r,
        }?;
        me.inner.update(inner_buf, false)?;
        let len = inner_buf.len();
        me.device.as_mut().consume(len);
        Poll::Ready(Ok(()))
    }
}

impl<R: AsyncBufRead + Unpin> AsyncRead for AsyncBufReadCompressor<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        if let Poll::Pending = Pin::new(&mut *self).fill_buf(cx)? {
            Poll::Pending
        } else {
            let me = self.project();
            let len = cmp::min(buf.len(), me.inner.buf().len() - *me.consumed);
            buf[..len].copy_from_slice(&me.inner.buf()[*me.consumed..][..len]);
            *me.consumed += len;
            if *me.consumed >= me.inner.buf().len() {
                me.inner.clear_buf();
                *me.consumed = 0;
            }
            Poll::Ready(Ok(len))
        }
    }
}

impl<R: AsyncBufRead + Unpin> AsyncBufRead for AsyncBufReadCompressor<R> {
    fn poll_fill_buf(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<&[u8]>> {
        if let Poll::Pending = Pin::new(&mut *self).fill_buf(cx)? {
            Poll::Pending
        } else {
            let me = self.project();
            Poll::Ready(Ok(&me.inner.buf()[*me.consumed..]))
        }
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

impl<R: AsyncBufRead + Unpin> TryInto<AsyncBufReadCompressor<R>> for CompressorBuilder<R> {
    type Error = crate::Error;
    fn try_into(self) -> crate::Result<AsyncBufReadCompressor<R>> {
        AsyncBufReadCompressor::from_builder(self.device, self.pref, self.dict)
    }
}
