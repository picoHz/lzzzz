#![cfg(feature = "tokio-io")]

use super::{AsyncBufReadCompressor, CompressorBuilder};
use pin_project::pin_project;
use std::{
    convert::TryInto,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, BufReader, Result};

#[cfg_attr(docsrs, doc(cfg(feature = "tokio-io")))]
#[pin_project]
pub struct AsyncReadCompressor<R: AsyncRead + Unpin> {
    #[pin]
    inner: AsyncBufReadCompressor<BufReader<R>>,
}

impl<R: AsyncRead + Unpin> AsyncRead for AsyncReadCompressor<R> {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context, buf: &mut [u8]) -> Poll<Result<usize>> {
        let mut me = self.project();
        me.inner.poll_read(cx, buf)
    }
}

impl<R: AsyncRead + Unpin> TryInto<AsyncReadCompressor<R>> for CompressorBuilder<R> {
    type Error = crate::Error;
    fn try_into(self) -> crate::Result<AsyncReadCompressor<R>> {
        Ok(AsyncReadCompressor {
            inner: AsyncBufReadCompressor::new(BufReader::new(self.device), self.pref, self.dict)?,
        })
    }
}
