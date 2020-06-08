#![allow(unsafe_code)]

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum FrameType {
    Frame = 0,
    SkippableFrame = 1,
}
