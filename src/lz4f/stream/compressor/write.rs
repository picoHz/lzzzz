use super::{Compressor, Dictionary, Preferences, State, LZ4F_HEADER_SIZE_MAX};
use std::{
    io::{IoSlice, Result, Write},
    mem::MaybeUninit,
};

pub struct WriteCompressor<W: Write> {
    inner: Compressor<W>,
    buffer: Vec<u8>,
}

impl<W: Write> WriteCompressor<W> {
    pub fn new(writer: W, pref: Preferences) -> Result<Self> {
        Ok(Self {
            inner: Compressor::new(writer, pref, None)?,
            buffer: Vec::new(),
        })
    }

    pub fn with_dict(writer: W, pref: Preferences, dict: Dictionary) -> Result<Self> {
        Ok(Self {
            inner: Compressor::new(writer, pref, Some(dict))?,
            buffer: Vec::new(),
        })
    }

    fn resize_buf(&mut self, src_size: usize) {
        let len = self.inner.compress_bound(src_size);
        if len > self.buffer.len() {
            self.buffer.reserve(len - self.buffer.len());

            #[allow(unsafe_code)]
            unsafe {
                self.buffer.set_len(len)
            };
        }
    }

    fn ensure_stream(&mut self) -> Result<()> {
        if let State::Created = self.inner.state {
            #[allow(unsafe_code, clippy::uninit_assumed_init)]
            let mut header =
                unsafe { [MaybeUninit::<u8>::uninit().assume_init(); LZ4F_HEADER_SIZE_MAX] };
            let header_len = self.inner.ctx.begin(&mut header[..], &self.inner.pref)?;
            self.inner.device.write_all(&header[..header_len])?;
            self.inner.state = State::WriteActive;
        }
        Ok(())
    }

    fn write_impl(&mut self, buf: &[u8], stable_src: bool) -> Result<usize> {
        self.ensure_stream()?;
        self.resize_buf(buf.len());
        let len = self.inner.ctx.update(&mut self.buffer, buf, stable_src)?;
        self.inner.device.write_all(&self.buffer[..len])?;
        Ok(buf.len())
    }

    fn end(&mut self) -> Result<()> {
        self.ensure_stream()?;
        match self.inner.state {
            State::WriteActive => {
                self.resize_buf(0);
                let len = self.inner.ctx.end(&mut self.buffer, false)?;
                self.inner.device.write_all(&self.buffer[..len])?;
            }
            _ => unreachable!(),
        }
        self.inner.device.flush()
    }
}

impl<W: Write> Write for WriteCompressor<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.write_impl(buf, false)
    }

    // fn write_vectored(&mut self, bufs: &[IoSlice]) -> Result<usize> {
    // let mut len = 0;
    // for (i, buf) in bufs.iter().enumerate() {
    // let is_last = i + 1 < buf.len();
    // len += self.write_impl(buf, !is_last)?;
    // }
    // Ok(len)
    // }

    fn flush(&mut self) -> Result<()> {
        self.ensure_stream()?;
        match self.inner.state {
            State::WriteActive => {
                self.resize_buf(0);
                let len = self.inner.ctx.flush(&mut self.buffer, false)?;
                self.inner.device.write_all(&self.buffer[..len])?;
            }
            _ => unreachable!(),
        }
        self.inner.device.flush()
    }
}

impl<W: Write> Drop for WriteCompressor<W> {
    fn drop(&mut self) {
        let _ = self.end();
    }
}
