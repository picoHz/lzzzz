#![cfg(feature = "tokio-io")]

use super::{Compressor, CompressorBuilder, Dictionary, Preferences, State, LZ4F_HEADER_SIZE_MAX};
use futures::future::FutureExt;
use pin_project::{pin_project, project};
use std::{
    convert::TryInto,
    marker::Unpin,
    mem::MaybeUninit,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncWrite, AsyncWriteExt, Result};

#[cfg_attr(docsrs, doc(cfg(feature = "tokio-io")))]
#[pin_project]
pub struct AsyncWriteCompressor<W: AsyncWrite> {
    #[pin]
    dev: W,
}
