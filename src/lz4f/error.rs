use std::{convert, fmt, io};

/// Errors from liblz4
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    Generic,
    MaxBlockSizeInvalid,
    BlockModeInvalid,
    ContentChecksumFlagInvalid,
    CompressionLevelInvalid,
    HeaderVersionWrong,
    BlockChecksumInvalid,
    ReservedFlagSet,
    AllocationFailed,
    SrcSizeTooLarge,
    DstMaxSizeTooSmall,
    FrameHeaderIncomplete,
    FrameTypeUnknown,
    FrameSizeWrong,
    SrcPtrWrong,
    DecompressionFailed,
    HeaderChecksumInvalid,
    ContentChecksumInvalid,
    FrameDecodingAlreadyStarted,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Error {
    Lz4f(ErrorKind),
    Common(crate::ErrorKind),
}

impl Error {
    pub(super) const fn new(kind: ErrorKind) -> Self {
        Self::Lz4f(kind)
    }
}

impl convert::From<Error> for io::Error {
    fn from(err: Error) -> Self {
        Self::new(io::ErrorKind::Other, err)
    }
}

impl convert::From<crate::Error> for Error {
    fn from(err: crate::Error) -> Self {
        Self::Common(err.kind())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        <Self as fmt::Debug>::fmt(self, f)
    }
}

impl std::error::Error for Error {}

/// A specialized Result type for compression/decompression operations.
pub type Result<T> = std::result::Result<T, Error>;
