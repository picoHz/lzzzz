mod api;
use std::{
    convert, fmt, io,
    rc::{Rc, Weak},
};

pub use api::{version_number, version_string};

/// A result of successful compression/decompression
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Report {
    pub(crate) dst_len: usize,
    pub(crate) src_len: Option<usize>,
    pub(crate) expected_len: Option<usize>,
}

impl Report {
    /// Return the length of the data written to the destination buffer.
    pub const fn dst_len(&self) -> usize {
        self.dst_len
    }

    /// Return the length of the data consumed from the source buffer.
    ///
    /// The value is present only if the underlying liblz4 API
    /// explicitly returns one.
    /// In most cases, the consumed length must be equal to the length of the source buffer
    /// and this method just returns [`None`].
    ///
    /// [`None`]: https://doc.rust-lang.org/nightly/core/option/enum.Option.html#variant.None
    pub const fn src_len(&self) -> Option<usize> {
        self.src_len
    }

    pub(crate) const fn expected_len(&self) -> Option<usize> {
        self.expected_len
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Compression/Decompression error
#[derive(Debug)]
pub enum Error {
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
    Unspecified,
    IOError(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        <Self as fmt::Debug>::fmt(self, f)
    }
}
impl convert::From<Error> for io::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::IOError(err) => err,
            _ => io::Error::new(io::ErrorKind::Other, err),
        }
    }
}

impl convert::From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::IOError(err)
    }
}

impl convert::From<usize> for Error {
    fn from(value: usize) -> Self {
        match value.wrapping_neg() {
            1 => Self::Generic,
            2 => Self::MaxBlockSizeInvalid,
            3 => Self::BlockModeInvalid,
            4 => Self::ContentChecksumFlagInvalid,
            5 => Self::CompressionLevelInvalid,
            6 => Self::HeaderVersionWrong,
            7 => Self::BlockChecksumInvalid,
            8 => Self::ReservedFlagSet,
            9 => Self::AllocationFailed,
            10 => Self::SrcSizeTooLarge,
            11 => Self::DstMaxSizeTooSmall,
            12 => Self::FrameHeaderIncomplete,
            13 => Self::FrameTypeUnknown,
            14 => Self::FrameSizeWrong,
            15 => Self::SrcPtrWrong,
            16 => Self::DecompressionFailed,
            17 => Self::HeaderChecksumInvalid,
            18 => Self::ContentChecksumInvalid,
            19 => Self::FrameDecodingAlreadyStarted,
            _ => Self::Unspecified,
        }
    }
}

impl std::error::Error for Error {}

pub(crate) use api::result_from_code;
