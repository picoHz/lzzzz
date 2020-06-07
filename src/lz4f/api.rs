#![allow(unsafe_code)]

use super::{AutoFlush, FavorDecSpeed, FrameInfo};
use libc::{c_int, c_uint};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum FrameType {
    Frame = 0,
    SkippableFrame = 1,
}
