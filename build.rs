fn main() -> Result<(), cc::Error> {
    let files = &["lz4hc.c", "lz4frame.c", "xxhash.c", "lz4.c"][..];
    let dir = std::path::Path::new("./vendor/liblz4/lib/");
    cc::Build::new()
        .files(files.iter().map(|file| dir.join(file)))
        .try_compile("lz4")
}
