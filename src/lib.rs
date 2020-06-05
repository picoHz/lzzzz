#![deny(unsafe_code)]

pub mod api;

use std::fmt;
use std::io;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct LZ4Error(&'static str);

impl fmt::Display for LZ4Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> { 
        write!(f, "{}", self.0)
     }
}

impl std::error::Error for LZ4Error {}

impl LZ4Error {
    pub(crate) fn new(msg: &'static str) -> Self {
        Self(msg)
    }
}

impl Into<io::Error> for LZ4Error {
    fn into(self) -> io::Error { 
        io::Error::new(io::ErrorKind::Other, self)
     }
}

type Result<T> = std::result::Result<T, LZ4Error>;
