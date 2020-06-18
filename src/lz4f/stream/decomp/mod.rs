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
    lz4f::{
        api::{
            header_size, DecompressionContext, LZ4F_HEADER_SIZE_MAX,
            LZ4F_MIN_SIZE_TO_KNOW_HEADER_LENGTH,
        },
        FrameInfo,
    },
    Error, LZ4Error, Report, Result,
};
use std::borrow::Cow;
use std::mem;
use std::mem::MaybeUninit;

enum State {
    Header {
        header: [u8; LZ4F_HEADER_SIZE_MAX],
        header_len: usize,
    },
    Body {
        frame_info: FrameInfo,
        comp_dict: *const u8,
    },
}

pub(crate) struct Decompressor<'a> {
    ctx: DecompressionContext,
    state: State,
    buffer: Vec<u8>,
    dict: Cow<'a, [u8]>,
}

impl<'a> Decompressor<'a> {
    pub fn new() -> Result<Self> {
        #[allow(unsafe_code)]
        Ok(Self {
            ctx: DecompressionContext::new()?,
            state: State::Header {
                header: unsafe {
                    mem::transmute(
                        MaybeUninit::<[MaybeUninit<u8>; LZ4F_HEADER_SIZE_MAX]>::uninit()
                            .assume_init(),
                    )
                },
                header_len: 0,
            },
            buffer: Vec::new(),
            dict: Cow::Borrowed(&[]),
        })
    }

    pub fn set_dict(&mut self, dict: Cow<'a, [u8]>) {
        self.dict = dict;
    }

    pub fn frame_info(&self) -> Option<FrameInfo> {
        if let State::Body { frame_info, .. } = self.state {
            Some(frame_info)
        } else {
            None
        }
    }

    pub fn decompress(&mut self, src: &[u8]) -> Result<Report> {
        let mut header_consumed = 0;
        if let State::Header {
            mut header,
            mut header_len,
        } = &mut self.state
        {
            if header_len < LZ4F_MIN_SIZE_TO_KNOW_HEADER_LENGTH {
                let len =
                    std::cmp::min(LZ4F_MIN_SIZE_TO_KNOW_HEADER_LENGTH - header_len, src.len());
                (&mut header[header_len..header_len + len]).copy_from_slice(&src[..len]);
                header_len += len;
                header_consumed += len;
            }
            if header_len >= LZ4F_MIN_SIZE_TO_KNOW_HEADER_LENGTH {
                let src = &src[header_consumed..];
                let exact_header_len = header_size(&header[..header_len]);
                if header_len < exact_header_len {
                    let len = std::cmp::min(exact_header_len - header_len, src.len());
                    (&mut header[header_len..header_len + len]).copy_from_slice(&src[..len]);
                    header_len += len;
                    header_consumed += len;
                }
                if header_len >= exact_header_len {
                    let (frame, rep) = self.ctx.get_frame_info(&header[..header_len])?;
                    header_consumed = std::cmp::min(header_consumed, rep);
                    self.state = State::Body {
                        frame_info: frame,
                        comp_dict: self.dict.as_ptr(),
                    }
                }
            }
            if src.is_empty() {
                self.ctx.get_frame_info(&header[..header_len])?;
            }
        }

        let src = &src[header_consumed..];
        if let State::Body { comp_dict, .. } = self.state {
            if self.dict.as_ptr() != comp_dict {
                return Err(Error::DictionaryChangedDuringDecompression.into());
            }

            let len = self.buffer.len();
            self.buffer.reserve(1024);
            #[allow(unsafe_code)]
            unsafe {
                self.buffer.set_len(self.buffer.capacity());
            }
            let report =
                self.ctx
                    .decompress_dict(src, &mut self.buffer[len..], &self.dict, false)?;
            self.buffer
                .resize_with(len + report.dst_len(), Default::default);
            Ok(Report {
                src_len: report.src_len.map(|len| len + header_consumed),
                ..report
            })
        } else {
            Ok(Report {
                src_len: Some(header_consumed),
                ..Default::default()
            })
        }
    }

    pub fn buf(&self) -> &[u8] {
        &self.buffer
    }

    pub fn clear_buf(&mut self) {
        self.buffer.clear();
    }
}
