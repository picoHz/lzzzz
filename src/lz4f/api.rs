#![allow(unsafe_code)]

use super::{
    binding, AutoFlush, BlockChecksum, BlockMode, BlockSize, ContentChecksum, FavorDecSpeed,
    FrameInfo, Preferences,
};
use crate::{LZ4Error, Result};

use binding::{CompressionCtx, CompressionDict, DecompressionCtx};
use libc::{c_int, c_uint, c_ulonglong, c_void, size_t};
use std::{
    ffi::CStr,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum FrameType {
    Frame = 0,
    SkippableFrame = 1,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Pref {
    pub frame_info: FrameInfo,
    pub compression_level: c_int,
    pub auto_flush: AutoFlush,
    pub favor_dec_speed: FavorDecSpeed,
    _reserved: [c_uint; 3],
}

impl Default for Pref {
    fn default() -> Self {
        Self {
            frame_info: FrameInfo::default(),
            compression_level: 0,
            auto_flush: AutoFlush::Disabled,
            favor_dec_speed: FavorDecSpeed::Disabled,
            _reserved: [0; 3],
        }
    }
}
