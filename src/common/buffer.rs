use std::{
    borrow::{Borrow, Cow},
    cmp,
    iter::FromIterator,
    ops::Deref,
};

/// Byte buffer smart pointer
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Buffer<'a> {
    Borrowed(&'a [u8]),
    Owned(Box<[u8]>),
    #[cfg(feature = "bytes")]
    #[cfg_attr(docsrs, doc(cfg(feature = "bytes")))]
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
            #[cfg(feature = "bytes")]
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

impl<'a> From<&'a Box<[u8]>> for Buffer<'a> {
    fn from(v: &'a Box<[u8]>) -> Self {
        Self::Borrowed(&v)
    }
}

impl From<Vec<u8>> for Buffer<'_> {
    fn from(v: Vec<u8>) -> Self {
        Self::Owned(v.into())
    }
}

impl<'a> From<&'a Vec<u8>> for Buffer<'a> {
    fn from(v: &'a Vec<u8>) -> Self {
        Self::Borrowed(v)
    }
}

impl FromIterator<u8> for Buffer<'_> {
    fn from_iter<T: IntoIterator<Item = u8>>(into_iter: T) -> Self {
        Vec::from_iter(into_iter).into()
    }
}

#[cfg(feature = "bytes")]
impl From<bytes::Bytes> for Buffer<'_> {
    fn from(v: bytes::Bytes) -> Self {
        Self::Bytes(v)
    }
}

#[cfg(feature = "bytes")]
impl From<bytes::BytesMut> for Buffer<'_> {
    fn from(v: bytes::BytesMut) -> Self {
        Self::Bytes(v.into())
    }
}

impl PartialEq<[u8]> for Buffer<'_> {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_ref() == other
    }
}

impl PartialOrd<[u8]> for Buffer<'_> {
    fn partial_cmp(&self, other: &[u8]) -> Option<cmp::Ordering> {
        self.as_ref().partial_cmp(other)
    }
}

impl PartialEq<Buffer<'_>> for [u8] {
    fn eq(&self, other: &Buffer<'_>) -> bool {
        *other == *self
    }
}

impl PartialOrd<Buffer<'_>> for [u8] {
    fn partial_cmp(&self, other: &Buffer<'_>) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self, other)
    }
}

impl PartialEq<Vec<u8>> for Buffer<'_> {
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.as_ref() == &other[..]
    }
}

impl PartialOrd<Vec<u8>> for Buffer<'_> {
    fn partial_cmp(&self, other: &Vec<u8>) -> Option<cmp::Ordering> {
        self.as_ref().partial_cmp(&other[..])
    }
}

impl PartialEq<Buffer<'_>> for Vec<u8> {
    fn eq(&self, other: &Buffer<'_>) -> bool {
        *other == *self
    }
}

impl PartialOrd<Buffer<'_>> for Vec<u8> {
    fn partial_cmp(&self, other: &Buffer<'_>) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self, other)
    }
}

#[cfg(feature = "bytes")]
impl PartialEq<bytes::Bytes> for Buffer<'_> {
    fn eq(&self, other: &bytes::Bytes) -> bool {
        self.as_ref() == &other[..]
    }
}

#[cfg(feature = "bytes")]
impl PartialOrd<bytes::Bytes> for Buffer<'_> {
    fn partial_cmp(&self, other: &bytes::Bytes) -> Option<cmp::Ordering> {
        self.as_ref().partial_cmp(&other[..])
    }
}

#[cfg(feature = "bytes")]
impl PartialEq<Buffer<'_>> for bytes::Bytes {
    fn eq(&self, other: &Buffer<'_>) -> bool {
        *other == *self
    }
}

#[cfg(feature = "bytes")]
impl PartialOrd<Buffer<'_>> for bytes::Bytes {
    fn partial_cmp(&self, other: &Buffer<'_>) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self, other)
    }
}

impl PartialEq<Buffer<'_>> for &[u8] {
    fn eq(&self, other: &Buffer<'_>) -> bool {
        other.as_ref() == *self
    }
}

impl PartialOrd<Buffer<'_>> for &[u8] {
    fn partial_cmp(&self, other: &Buffer<'_>) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self, other)
    }
}

impl<'a, T: ?Sized> PartialEq<&'a T> for Buffer<'a>
where
    Buffer<'a>: PartialEq<T>,
{
    fn eq(&self, other: &&'a T) -> bool {
        *self == **other
    }
}

impl<'a, T: ?Sized> PartialOrd<&'a T> for Buffer<'a>
where
    Buffer<'a>: PartialOrd<T>,
{
    fn partial_cmp(&self, other: &&'a T) -> Option<cmp::Ordering> {
        self.partial_cmp(&**other)
    }
}
