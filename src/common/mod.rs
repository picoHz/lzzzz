#![allow(unsafe_code)]

mod binding;

pub fn version_number() -> u32 {
    unsafe { binding::LZ4_versionNumber() as u32 }
}
