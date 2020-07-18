#![cfg(feature = "tokio-io")]

use super::{Compressor, Dictionary, Preferences};
use crate::lz4f::Result;
use pin_project::pin_project;
use std::{
    cmp,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncBufRead, AsyncRead};

/// The [`AsyncBufRead`]-based streaming compressor.
///
/// # Example
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
/// use lzzzz::lz4f::AsyncBufReadCompressor;
/// use tokio::{fs::File, io::BufReader, prelude::*};
///
/// let mut f = File::open("foo.txt").await?;
/// let mut b = BufReader::new(f);
/// let mut r = AsyncBufReadCompressor::new(&mut b, Default::default())?;
///
/// let mut buf = Vec::new();
/// r.read_to_end(&mut buf).await?;
/// # Ok::<(), tokio::io::Error>(())
/// # }).unwrap();
/// # tmp_dir.close().unwrap();
/// ```
///
/// [`AsyncBufRead`]: https://docs.rs/tokio/latest/tokio/io/trait.AsyncBufRead.html

#[cfg_attr(docsrs, doc(cfg(feature = "tokio-io")))]
#[pin_project]
pub struct AsyncBufReadCompressor<R: AsyncBufRead + Unpin> {
    #[pin]
    device: R,
    inner: Compressor,
    consumed: usize,
    closed: bool,
    state: State,
}

enum State {
    None,
    Read,
    FillBuf,
}

impl<R: AsyncBufRead + Unpin> AsyncBufReadCompressor<R> {
    /// Creates a new `AsyncBufReadCompressor<R>`.
    pub fn new(reader: R, prefs: Preferences) -> Result<Self> {
        Ok(Self {
            device: reader,
            inner: Compressor::new(prefs, None)?,
            consumed: 0,
            closed: false,
            state: State::None,
        })
    }

    /// Creates a new `AsyncBufReadCompressor<R>` with a dictionary.
    pub fn with_dict(reader: R, prefs: Preferences, dict: Dictionary) -> Result<Self> {
        Ok(Self {
            device: reader,
            inner: Compressor::new(prefs, Some(dict))?,
            consumed: 0,
            closed: false,
            state: State::None,
        })
    }

    fn fill_buf(self: Pin<&mut Self>, cx: &mut Context) -> Poll<tokio::io::Result<()>> {
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
        if inner_buf.is_empty() {
            if !*me.closed {
                me.inner.end(false)?;
                *me.closed = true;
            }
        } else {
            me.inner.update(inner_buf, false)?;
        }
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
    ) -> Poll<tokio::io::Result<usize>> {
        let mut me = Pin::new(&mut *self);
        if let State::None = me.state {
            me.state = State::Read;
        }
        if let State::Read = me.state {
            if let Poll::Ready(r) = me.fill_buf(cx) {
                let me = self.project();
                *me.state = State::None;
                r?;
                let len = cmp::min(buf.len(), me.inner.buf().len() - *me.consumed);
                buf[..len].copy_from_slice(&me.inner.buf()[*me.consumed..][..len]);
                *me.consumed += len;
                if *me.consumed >= me.inner.buf().len() {
                    me.inner.clear_buf();
                    *me.consumed = 0;
                }
                return Poll::Ready(Ok(len));
            }
        }
        Poll::Pending
    }
}

impl<R: AsyncBufRead + Unpin> AsyncBufRead for AsyncBufReadCompressor<R> {
    fn poll_fill_buf(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<tokio::io::Result<&[u8]>> {
        let mut me = Pin::new(&mut *self);
        if let State::None = me.state {
            me.state = State::FillBuf;
        }
        if let State::FillBuf = me.state {
            if let Poll::Ready(r) = me.fill_buf(cx) {
                let me = self.project();
                *me.state = State::None;
                r?;
                return Poll::Ready(Ok(&me.inner.buf()[*me.consumed..]));
            }
        }
        Poll::Pending
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
