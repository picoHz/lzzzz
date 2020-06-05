#![deny(unsafe_code)]

pub mod lz4f;
pub mod lz4hc;

use std::{convert, fmt, io};

#[derive(Debug)]
pub enum LZ4Error {
    LZ4(&'static str),
    IO(io::Error),
}

impl fmt::Display for LZ4Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        match self {
            Self::LZ4(err) => write!(f, "{}", err),
            Self::IO(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for LZ4Error {}

impl From<&'static str> for LZ4Error {
    fn from(err: &'static str) -> Self {
        Self::LZ4(err)
    }
}

impl convert::From<io::Error> for LZ4Error {
    fn from(err: io::Error) -> Self {
        Self::IO(err)
    }
}

impl convert::From<LZ4Error> for io::Error {
    fn from(err: LZ4Error) -> Self {
        match err {
            LZ4Error::LZ4(err) => io::Error::new(io::ErrorKind::Other, err),
            LZ4Error::IO(err) => err,
        }
    }
}

type Result<T> = std::result::Result<T, LZ4Error>;
