

#[link(name = "lz4")]
extern "C" {
    pub fn LZ4_versionNumber() -> libc::c_int;
}