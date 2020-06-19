#![cfg(feature = "use-tokio")]

use super::{AsyncBufReadCompressor, Dictionary, Preferences};
use crate::lz4f::CompressorBuilder;
use pin_project::pin_project;
use std::{
    convert::TryInto,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, BufReader, Result};

/// AsyncRead-based streaming compressor
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
/// use lzzzz::lz4f::comp::AsyncReadCompressor;
/// use tokio::{fs::File, prelude::*};
///
/// let mut f = File::open("foo.txt").await?;
/// let mut r = AsyncReadCompressor::new(&mut f)?;
///
/// let mut buf = Vec::new();
/// r.read_to_end(&mut buf).await?;
/// # Ok::<(), tokio::io::Error>(())
/// # }).unwrap();
/// # tmp_dir.close().unwrap();
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "use-tokio")))]
#[pin_project]
pub struct AsyncReadCompressor<R: AsyncRead + Unpin> {
    #[pin]
    inner: AsyncBufReadCompressor<BufReader<R>>,
}

impl<R: AsyncRead + Unpin> AsyncReadCompressor<R> {
    pub fn new(reader: R) -> crate::Result<Self> {
        Self::from_builder(reader, Default::default(), None)
    }

    fn from_builder(reader: R, pref: Preferences, dict: Option<Dictionary>) -> crate::Result<Self> {
        Ok(Self {
            inner: AsyncBufReadCompressor::from_builder(BufReader::new(reader), pref, dict)?,
        })
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for AsyncReadCompressor<R> {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context, buf: &mut [u8]) -> Poll<Result<usize>> {
        self.project().inner.poll_read(cx, buf)
    }
}

impl<R: AsyncRead + Unpin> TryInto<AsyncReadCompressor<R>> for CompressorBuilder<R> {
    type Error = crate::Error;
    fn try_into(self) -> crate::Result<AsyncReadCompressor<R>> {
        AsyncReadCompressor::from_builder(self.device, self.pref, self.dict)
    }
}
