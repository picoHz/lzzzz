#![allow(unsafe_code)]

use super::Result;
use crate::binding;
use std::ffi::CStr;

/// Returns the version number of liblz4.
///
/// # Example
///
/// ```
/// assert_eq!(lzzzz::version_number(), 10902); // 1.9.2
/// ```
pub fn version_number() -> u32 {
    unsafe { binding::LZ4_versionNumber() as u32 }
}

/// Returns the version string of liblz4.
///
/// # Example
///
/// ```
/// assert_eq!(lzzzz::version_string(), "1.9.2");
/// ```
pub fn version_string() -> &'static str {
    unsafe {
        CStr::from_ptr(binding::LZ4_versionString())
            .to_str()
            .unwrap()
    }
}

pub fn result_from_code(code: usize) -> Result<()> {
    if unsafe { binding::LZ4F_isError(code) } == 0 {
        Ok(())
    } else {
        Err(code.into())
    }
}
