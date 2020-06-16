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

#[cfg_attr(docsrs, doc(cfg(feature = "tokio-io")))]
#[pin_project]
pub struct AsyncBufReadCompressor<'a, B: AsyncBufRead + Unpin> {
    #[pin]
    device: B,
    inner: Decompressor<'a>,
    consumed: usize,
}

impl<'a, B: AsyncBufRead + Unpin> AsyncBufReadCompressor<'a, B> {
    pub(crate) fn new(device: B) -> crate::Result<Self> {
        Ok(Self {
            device,
            inner: Decompressor::new()?,
            consumed: 0,
        })
    }

    pub fn set_dict(&mut self, dict: Cow<'a, [u8]>) {
        self.inner.set_dict(dict);
    }

    pub async fn read_frame_info(&mut self) -> Result<FrameInfo> {
        todo!()
    }
}

impl<'a, B: AsyncBufRead + Unpin> AsyncRead for AsyncBufReadCompressor<'a, B> {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context, buf: &mut [u8]) -> Poll<Result<usize>> {
        todo!();
    }
}

impl<'a, B: AsyncBufRead + Unpin> TryInto<AsyncBufReadCompressor<'a, B>>
    for DecompressorBuilder<B>
{
    type Error = crate::Error;
    fn try_into(self) -> crate::Result<AsyncBufReadCompressor<'a, B>> {
        AsyncBufReadCompressor::new(self.device)
    }
}
