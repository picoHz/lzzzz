#![cfg(feature = "tokio-io")]

use super::AsyncBufReadCompressor;
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
use tokio::io::{AsyncRead, BufReader, Result};

/// AsyncRead-based streaming decompressor
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-io")))]
#[pin_project]
pub struct AsyncReadCompressor<'a, R: AsyncRead + Unpin> {
    #[pin]
    inner: AsyncBufReadCompressor<'a, BufReader<R>>,
}

impl<'a, R: AsyncRead + Unpin> AsyncReadCompressor<'a, R> {
    pub fn new(reader: R) -> crate::Result<Self> {
        Self::from_builder(reader)
    }

    fn from_builder(device: R) -> crate::Result<Self> {
        Ok(Self {
            inner: AsyncBufReadCompressor::from_builder(BufReader::new(device))?,
        })
    }

    pub fn set_dict(&mut self, dict: Cow<'a, [u8]>) {
        self.inner.set_dict(dict);
    }

    pub async fn read_frame_info(&mut self) -> Result<FrameInfo> {
        self.inner.read_frame_info().await
    }
}

impl<'a, R: AsyncRead + Unpin> AsyncRead for AsyncReadCompressor<'a, R> {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context, buf: &mut [u8]) -> Poll<Result<usize>> {
        self.project().inner.poll_read(cx, buf)
    }
}

impl<'a, R: AsyncRead + Unpin> TryInto<AsyncReadCompressor<'a, R>> for DecompressorBuilder<R> {
    type Error = crate::Error;
    fn try_into(self) -> crate::Result<AsyncReadCompressor<'a, R>> {
        AsyncReadCompressor::from_builder(self.device)
    }
}
