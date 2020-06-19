use super::BufReadDecompressor;
use crate::{
    lz4f::{DecompressorBuilder, FrameInfo},
    Buffer,
};
use std::{
    convert::TryInto,
    io::{BufReader, Read, Result},
};

/// Read-based streaming decompressor
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
/// # let mut buf = Vec::new();
/// # lzzzz::lz4f::compress_to_vec(b"Hello world!", &mut buf, &Default::default())?;
/// # tmp_dir.child("foo.lz4").write_binary(&buf).unwrap();
/// #
/// use lzzzz::lz4f::decomp::ReadDecompressor;
/// use std::{fs::File, io::prelude::*};
///
/// let mut f = File::open("foo.lz4")?;
/// let mut r = ReadDecompressor::new(&mut f)?;
///
/// let mut buf = Vec::new();
/// r.read_to_end(&mut buf)?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub struct ReadDecompressor<'a, R: Read> {
    inner: BufReadDecompressor<'a, BufReader<R>>,
}

impl<'a, R: Read> ReadDecompressor<'a, R> {
    pub fn new(reader: R) -> crate::Result<Self> {
        Self::from_builder(reader)
    }

    fn from_builder(device: R) -> crate::Result<Self> {
        Ok(Self {
            inner: BufReadDecompressor::from_builder(BufReader::new(device))?,
        })
    }

    pub fn read_frame_info(&mut self) -> Result<FrameInfo> {
        self.inner.read_frame_info()
    }

    pub fn set_dict<B>(&mut self, dict: B)
    where
        B: Into<Buffer<'a>>,
    {
        self.inner.set_dict(dict.into());
    }
}

impl<R: Read> Read for ReadDecompressor<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.inner.read(buf)
    }
}

impl<'a, R: Read> TryInto<ReadDecompressor<'a, R>> for DecompressorBuilder<R> {
    type Error = crate::Error;
    fn try_into(self) -> crate::Result<ReadDecompressor<'a, R>> {
        ReadDecompressor::from_builder(self.device)
    }
}
