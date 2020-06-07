#![allow(unsafe_code)]

mod binding;

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
