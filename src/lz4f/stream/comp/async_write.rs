#![cfg(feature = "async-io")]

use super::{Compressor, Dictionary, Preferences};
use crate::lz4f::Result;
use futures_lite::{ready, AsyncWrite};
use pin_project::pin_project;
use std::{
    fmt, io,
    marker::Unpin,
    pin::Pin,
    task::{Context, Poll},
};

pub(crate) const WRITER_INVALID_STATE: &str = "writer must be polled to completion";

/// The [`AsyncWrite`]-based stream compressor.
///
/// # Example
///
/// ```
/// # use std::env;
/// # use std::path::Path;
/// # use lzzzz::{Error, Result};
/// # let tmp_dir = assert_fs::TempDir::new().unwrap().into_persistent();
/// # env::set_current_dir(tmp_dir.path()).unwrap();
/// # smol::run(async {
/// use async_std::fs::File;
/// use futures_lite::*;
/// use lzzzz::lz4f::AsyncWriteCompressor;
///
/// let mut f = File::create("foo.lz4").await?;
/// let mut w = AsyncWriteCompressor::new(&mut f, Default::default())?;
///
/// w.write_all(b"Hello world!").await?;
///
/// // You have to call close() to finalize the frame.
/// w.close().await?;
/// # Ok::<(), std::io::Error>(())
/// # }).unwrap();
/// # tmp_dir.close().unwrap();
/// ```
///
/// [`AsyncWrite`]: https://docs.rs/futures-io/0.3.5/futures_io/trait.AsyncWrite.html

#[cfg_attr(docsrs, doc(cfg(feature = "async-io")))]
#[pin_project]
pub struct AsyncWriteCompressor<W: AsyncWrite + Unpin> {
    #[pin]
    inner: W,
    comp: Compressor,
    consumed: usize,
    state: State,
}

#[derive(Copy, Clone)]
enum State {
    None,
    Write(usize),
    WriteThenFlush,
    Flush,
    WriteThenClose,
    FlushThenClose,
    Close,
    Shutdown,
}

impl<W: AsyncWrite + Unpin> AsyncWriteCompressor<W> {
    /// Creates a new `AsyncWriteCompressor<W>`.
    pub fn new(writer: W, prefs: Preferences) -> Result<Self> {
        Ok(Self {
            inner: writer,
            comp: Compressor::new(prefs, None)?,
            consumed: 0,
            state: State::None,
        })
    }

    /// Creates a new `AsyncWriteCompressor<W>` with a dictionary.
    pub fn with_dict(writer: W, prefs: Preferences, dict: Dictionary) -> Result<Self> {
        Ok(Self {
            inner: writer,
            comp: Compressor::new(prefs, Some(dict))?,
            consumed: 0,
            state: State::None,
        })
    }

    /// Returns a mutable reference to the writer.
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }

    /// Returns a shared reference to the writer.
    pub fn get_ref(&self) -> &W {
        &self.inner
    }

    /// Returns ownership of the writer.
    pub fn into_inner(self) -> W {
        self.inner
    }

    fn poll_write_all(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        loop {
            let me = self.as_mut().project();
            let size = ready!(me.inner.poll_write(cx, &me.comp.buf()[*me.consumed..]))?;
            self.consumed += size;
            if self.consumed >= self.comp.buf().len() {
                debug_assert_eq!(self.consumed, self.comp.buf().len());
                self.consumed = 0;
                self.comp.clear_buf();
                return Poll::Ready(Ok(()));
            }
        }
    }

    fn poll_state(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<usize>> {
        loop {
            match self.state {
                State::Write(size) => {
                    ready!(self.as_mut().poll_write_all(cx))?;
                    self.state = State::None;
                    return Poll::Ready(Ok(size));
                }
                State::WriteThenFlush => {
                    ready!(self.as_mut().poll_write_all(cx))?;
                    self.state = State::Flush;
                }
                State::Flush => {
                    ready!(self.as_mut().project().inner.poll_flush(cx))?;
                    self.state = State::None;
                }
                State::WriteThenClose => {
                    ready!(self.as_mut().poll_write_all(cx))?;
                    self.state = State::FlushThenClose;
                }
                State::FlushThenClose => {
                    ready!(self.as_mut().project().inner.poll_flush(cx))?;
                    self.state = State::Close;
                }
                State::Close => {
                    ready!(self.as_mut().project().inner.poll_close(cx))?;
                    self.state = State::Shutdown;
                }
                State::None | State::Shutdown => return Poll::Ready(Ok(0)),
            }
        }
    }
}

impl<W> fmt::Debug for AsyncWriteCompressor<W>
where
    W: AsyncWrite + Unpin + fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("AsyncWriteCompressor")
            .field("writer", &self.inner)
            .field("prefs", &self.comp.prefs())
            .finish()
    }
}

impl<W: AsyncWrite + Unpin> AsyncWrite for AsyncWriteCompressor<W> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match self.state {
            State::None => {
                self.comp.update(buf, false)?;
                self.state = State::Write(buf.len());
            }
            State::Write(_) | State::Shutdown => (),
            _ => {
                let err = io::Error::new(io::ErrorKind::Other, WRITER_INVALID_STATE);
                return Poll::Ready(Err(err));
            }
        }

        self.poll_state(cx)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        match self.state {
            State::None => {
                self.comp.flush(false)?;
                self.state = State::WriteThenFlush;
            }
            State::WriteThenFlush | State::Flush | State::Shutdown => (),
            _ => {
                let err = io::Error::new(io::ErrorKind::Other, WRITER_INVALID_STATE);
                return Poll::Ready(Err(err));
            }
        }

        self.poll_state(cx).map_ok(|_| ())
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        match self.state {
            State::None => {
                self.comp.end(false)?;
                self.state = State::WriteThenClose;
            }
            State::WriteThenClose | State::FlushThenClose | State::Close | State::Shutdown => (),
            _ => {
                let err = io::Error::new(io::ErrorKind::Other, WRITER_INVALID_STATE);
                return Poll::Ready(Err(err));
            }
        }

        self.poll_state(cx).map_ok(|_| ())
    }
}
