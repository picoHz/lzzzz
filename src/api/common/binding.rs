use libc::c_int;

#[link(name = "lz4")]
extern "C" {
    pub fn LZ4_versionNumber() -> c_int;
    pub fn LZ4_compressBound(input_size: c_int) -> c_int;
}

#[cfg(test)]
mod tests {
    #[test]
    fn call_c_api() {
        unsafe {
            assert_eq!(super::LZ4_versionNumber(), 10902);
            assert_eq!(super::LZ4_compressBound(1000), 1019);
        }
    }
}
