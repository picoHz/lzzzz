#![cfg(feature = "async-io")]

use super::{super::comp::WRITER_INVALID_STATE, Decompressor};
use crate::lz4f::{FrameInfo, Result};
use futures_lite::{ready, AsyncWrite};
use pin_project::pin_project;
use std::{
    borrow::Cow,
    fmt, io,
    marker::Unpin,
    pin::Pin,
    task::{Context, Poll},
};

/// The [`AsyncWrite`]-based streaming decompressor.
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
/// use async_std::{fs::File, prelude::*};
/// use lzzzz::lz4f::{compress_to_vec, AsyncWriteDecompressor};
///
/// let mut f = File::create("foo.txt").await?;
/// let mut w = AsyncWriteDecompressor::new(&mut f)?;
///
/// let mut buf = Vec::new();
/// compress_to_vec(b"Hello world!", &mut buf, &Default::default())?;
///
/// w.write_all(&buf).await?;
/// # Ok::<(), std::io::Error>(())
/// # }).unwrap();
/// # tmp_dir.close().unwrap();
/// ```
///
/// [`AsyncWrite`]: https://docs.rs/futures-io/0.3.5/futures_io/trait.AsyncWrite.html

#[cfg_attr(docsrs, doc(cfg(feature = "async-io")))]
#[pin_project]
pub struct AsyncWriteDecompressor<'a, W: AsyncWrite + Unpin> {
    #[pin]
    inner: W,
    decomp: Decompressor<'a>,
    consumed: usize,
    state: State,
}

enum State {
    None,
    Write(usize),
    Flush,
    Close,
    Shutdown,
}

impl<'a, W: AsyncWrite + Unpin> AsyncWriteDecompressor<'a, W> {
    /// Creates a new `AsyncWriteDecompressor<W>`.
    pub fn new(writer: W) -> Result<Self> {
        Ok(Self {
            inner: writer,
            decomp: Decompressor::new()?,
            consumed: 0,
            state: State::None,
        })
    }

    /// Sets the dictionary.
    pub fn set_dict<D>(&mut self, dict: D)
    where
        D: Into<Cow<'a, [u8]>>,
    {
        self.decomp.set_dict(dict);
    }

    /// Returns `FrameInfo` if the frame header is already decoded.
    /// Otherwise, returns `None`.
    pub fn frame_info(&self) -> Option<FrameInfo> {
        self.decomp.frame_info()
    }

    /// Sets the 'header-only' mode.
    ///
    /// When the 'header-only' mode is enabled, the decompressor doesn't
    /// consume the frame body and `poll_write()` always returns `Ok(0)`
    /// if the frame header is already decoded.
    pub fn decode_header_only(&mut self, flag: bool) {
        self.decomp.decode_header_only(flag);
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
            let size = ready!(me.inner.poll_write(cx, &me.decomp.buf()[*me.consumed..]))?;
            self.consumed += size;
            if self.consumed >= self.decomp.buf().len() {
                debug_assert_eq!(self.consumed, self.decomp.buf().len());
                self.consumed = 0;
                self.decomp.clear_buf();
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
                State::Flush => {
                    ready!(self.as_mut().project().inner.poll_flush(cx))?;
                    self.state = State::None;
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

impl<W> fmt::Debug for AsyncWriteDecompressor<'_, W>
where
    W: AsyncWrite + Unpin + fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("AsyncWriteDecompressor")
            .field("writer", &self.inner)
            .finish()
    }
}

impl<W: AsyncWrite + Unpin> AsyncWrite for AsyncWriteDecompressor<'_, W> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match self.state {
            State::None => {
                self.state = State::Write(self.decomp.decompress(buf)?);
            }
            State::Write(_) | State::Shutdown => (),
            _ => {
                let err = io::Error::new(io::ErrorKind::Other, WRITER_INVALID_STATE);
                return Poll::Ready(Err(err));
            }
        }

        self.poll_state(cx)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.state {
            State::None => {
                self.state = State::Flush;
            }
            State::Flush | State::Shutdown => (),
            _ => {
                let err = io::Error::new(io::ErrorKind::Other, WRITER_INVALID_STATE);
                return Poll::Ready(Err(err));
            }
        }

        self.poll_state(cx).map_ok(|_| ())
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.state {
            State::None => {
                self.state = State::Close;
            }
            State::Close | State::Shutdown => (),
            _ => {
                let err = io::Error::new(io::ErrorKind::Other, WRITER_INVALID_STATE);
                return Poll::Ready(Err(err));
            }
        }

        self.poll_state(cx).map_ok(|_| ())
    }
}
