use std::{
    borrow::{Borrow, Cow},
    iter::FromIterator,
    ops::Deref,
};

/// Byte buffer smart pointer
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Buffer<'a> {
    Borrowed(&'a [u8]),
    Owned(Box<[u8]>),
    #[cfg(feature = "use-bytes")]
    #[cfg_attr(docsrs, doc(cfg(feature = "use-bytes")))]
    Bytes(bytes::Bytes),
}

impl Buffer<'_> {
    pub const fn new() -> Self {
        const EMPTY: &[u8] = &[];
        Self::Borrowed(EMPTY)
    }
}

impl Default for Buffer<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Deref for Buffer<'a> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        match self {
            Self::Borrowed(v) => v,
            Self::Owned(v) => v,
            #[cfg(feature = "use-bytes")]
            Self::Bytes(v) => v,
        }
    }
}

impl<'a> AsRef<[u8]> for Buffer<'a> {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl<'a> Borrow<[u8]> for Buffer<'a> {
    fn borrow(&self) -> &[u8] {
        self
    }
}

impl<'a> From<&'a [u8]> for Buffer<'a> {
    fn from(v: &'a [u8]) -> Self {
        Self::Borrowed(v)
    }
}

impl<'a> From<Cow<'a, [u8]>> for Buffer<'a> {
    fn from(v: Cow<'a, [u8]>) -> Self {
        match v {
            Cow::Borrowed(v) => Self::Borrowed(v),
            Cow::Owned(v) => Self::Owned(v.into()),
        }
    }
}

impl From<Box<[u8]>> for Buffer<'_> {
    fn from(v: Box<[u8]>) -> Self {
        Self::Owned(v)
    }
}

impl From<Vec<u8>> for Buffer<'_> {
    fn from(v: Vec<u8>) -> Self {
        Self::Owned(v.into())
    }
}

impl FromIterator<u8> for Buffer<'_> {
    fn from_iter<T: IntoIterator<Item = u8>>(into_iter: T) -> Self {
        Vec::from_iter(into_iter).into()
    }
}

#[cfg(feature = "use-bytes")]
impl From<bytes::Bytes> for Buffer<'_> {
    fn from(v: bytes::Bytes) -> Self {
        Self::Bytes(v)
    }
}

#[cfg(feature = "use-bytes")]
impl From<bytes::BytesMut> for Buffer<'_> {
    fn from(v: bytes::BytesMut) -> Self {
        Self::Bytes(v.into())
    }
}
