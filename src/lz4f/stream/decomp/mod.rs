//! Streaming Decompressors
mod async_bufread;
mod async_read;
mod async_write;
mod bufread;
mod read;
mod write;

pub use bufread::*;
pub use read::*;
pub use write::*;

#[cfg(feature = "tokio-io")]
pub use {async_bufread::*, async_read::*, async_write::*};

use crate::{
    common::DEFAULT_BUF_SIZE,
    lz4f::{
        api::{
            header_size, DecompressionContext, LZ4F_HEADER_SIZE_MAX,
            LZ4F_MIN_SIZE_TO_KNOW_HEADER_LENGTH,
        },
        FrameInfo, Result,
    },
    Buffer, Error, ErrorKind,
};
use std::{cmp, mem, mem::MaybeUninit, ptr};

enum State {
    Header {
        header: [u8; LZ4F_HEADER_SIZE_MAX],
        header_len: usize,
    },
    Body {
        frame_info: FrameInfo,
        comp_dict: Option<(*const u8, usize)>,
    },
}

pub(crate) struct Decompressor<'a> {
    ctx: DecompressionContext,
    state: State,
    buffer: Vec<u8>,
    dict: Buffer<'a>,
    header_only: bool,
}

impl<'a> Decompressor<'a> {
    pub fn new() -> Result<Self> {
        let header = MaybeUninit::<[MaybeUninit<u8>; LZ4F_HEADER_SIZE_MAX]>::uninit();
        #[allow(unsafe_code)]
        let header = unsafe { mem::transmute(header.assume_init()) };
        Ok(Self {
            ctx: DecompressionContext::new()?,
            state: State::Header {
                header,
                header_len: 0,
            },
            buffer: Vec::with_capacity(DEFAULT_BUF_SIZE),
            dict: Buffer::new(),
            header_only: false,
        })
    }

    pub fn set_dict(&mut self, dict: Buffer<'a>) {
        self.dict = dict;
    }

    pub fn frame_info(&self) -> Option<FrameInfo> {
        if let State::Body { frame_info, .. } = self.state {
            Some(frame_info)
        } else {
            None
        }
    }

    pub fn decode_header_only(&mut self, flag: bool) {
        self.header_only = flag;
    }

    pub fn decompress(&mut self, src: &[u8]) -> Result<usize> {
        let mut header_consumed = 0;
        if let State::Header {
            mut header,
            mut header_len,
        } = &mut self.state
        {
            if header_len < LZ4F_MIN_SIZE_TO_KNOW_HEADER_LENGTH {
                let len = cmp::min(LZ4F_MIN_SIZE_TO_KNOW_HEADER_LENGTH - header_len, src.len());
                (&mut header[header_len..header_len + len]).copy_from_slice(&src[..len]);
                header_len += len;
                header_consumed += len;
            }
            if header_len >= LZ4F_MIN_SIZE_TO_KNOW_HEADER_LENGTH {
                let src = &src[header_consumed..];
                let exact_header_len = header_size(&header[..header_len]);
                if exact_header_len > LZ4F_HEADER_SIZE_MAX {
                    // TODO
                    return Err(Error::new(ErrorKind::DecompressionFailed).into());
                }
                if header_len < exact_header_len {
                    let len = cmp::min(exact_header_len - header_len, src.len());
                    (&mut header[header_len..header_len + len]).copy_from_slice(&src[..len]);
                    header_len += len;
                    header_consumed += len;
                }
                if header_len >= exact_header_len {
                    let (frame, rep) = self.ctx.get_frame_info(&header[..header_len])?;
                    header_consumed = cmp::min(header_consumed, rep);

                    self.state = State::Body {
                        frame_info: frame,
                        comp_dict: None,
                    }
                }
            }
            if src.is_empty() {
                self.ctx.get_frame_info(&header[..header_len])?;
            }
        }

        if self.header_only {
            return Ok(header_consumed);
        }

        let src = &src[header_consumed..];
        let dict_ptr = self.dict_ptr();
        if let State::Body { comp_dict, .. } = &mut self.state {
            if dict_ptr != *comp_dict.get_or_insert(dict_ptr) {
                return Err(Error::new(ErrorKind::DictionaryChangedDuringDecompression).into());
            }

            let len = self.buffer.len();
            #[allow(unsafe_code)]
            unsafe {
                self.buffer.set_len(self.buffer.capacity());
            }
            let (src_len, dst_len, _) =
                self.ctx
                    .decompress_dict(src, &mut self.buffer[len..], &self.dict, false)?;
            self.buffer.resize_with(len + dst_len, Default::default);
            Ok(src_len + header_consumed)
        } else {
            Ok(header_consumed)
        }
    }

    fn dict_ptr(&self) -> (*const u8, usize) {
        if self.dict.is_empty() {
            (ptr::null(), 0)
        } else {
            (self.dict.as_ptr(), self.dict.len())
        }
    }

    pub fn buf(&self) -> &[u8] {
        &self.buffer
    }

    pub fn clear_buf(&mut self) {
        self.buffer.clear();
    }
}
