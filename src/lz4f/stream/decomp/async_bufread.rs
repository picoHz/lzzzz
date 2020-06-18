#![cfg(feature = "tokio-io")]

use super::Decompressor;
use crate::{
    common::LZ4Error,
    lz4f::{DecompressorBuilder, FrameInfo},
};
use pin_project::pin_project;
use std::{
    borrow::Cow,
    convert::TryInto,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncBufRead, AsyncRead, Result};

#[derive(PartialEq)]
enum State {
    None,
    Read,
    FillBuf,
}

/// AsyncBufRead-based streaming decompressor
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-io")))]
#[pin_project]
pub struct AsyncBufReadCompressor<'a, R: AsyncBufRead + Unpin> {
    #[pin]
    device: R,
    inner: Decompressor<'a>,
    state: State,
    consumed: usize,
}

impl<'a, R: AsyncBufRead + Unpin> AsyncBufReadCompressor<'a, R> {
    pub fn new(reader: R) -> crate::Result<Self> {
        Self::from_builder(reader)
    }

    pub(super) fn from_builder(device: R) -> crate::Result<Self> {
        Ok(Self {
            device,
            inner: Decompressor::new()?,
            state: State::None,
            consumed: 0,
        })
    }

    pub fn set_dict(&mut self, dict: Cow<'a, [u8]>) {
        self.inner.set_dict(dict);
    }

    pub async fn read_frame_info(&mut self) -> Result<FrameInfo> {
        todo!()
    }

    fn poll_read_impl(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
        state: State,
    ) -> Poll<Result<usize>> {
        todo!();
    }
}

impl<'a, R: AsyncBufRead + Unpin> AsyncRead for AsyncBufReadCompressor<'a, R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        let result = match Pin::new(&mut *self).poll_read_impl(cx, buf, State::Read) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(r) => r,
        };
        let me = self.project();
        *me.state = State::None;
        Poll::Ready(result)
    }
}

impl<'a, R: AsyncBufRead + Unpin> AsyncBufRead for AsyncBufReadCompressor<'a, R> {
    fn poll_fill_buf(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<&[u8]>> {
        let result = match Pin::new(&mut *self).poll_read_impl(cx, &mut [], State::FillBuf) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(r) => r,
        };
        let me = self.project();
        *me.state = State::None;
        result?;
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

impl<'a, R: AsyncBufRead + Unpin> TryInto<AsyncBufReadCompressor<'a, R>>
    for DecompressorBuilder<R>
{
    type Error = crate::Error;
    fn try_into(self) -> crate::Result<AsyncBufReadCompressor<'a, R>> {
        AsyncBufReadCompressor::new(self.device)
    }
}