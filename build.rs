use std::io::{Error, ErrorKind};

fn main() -> Result<(), Error> {
    cc::Build::new()
        .files(
            globwalk::glob("vendor/liblz4/lib/**/*.c")?
                .filter_map(Result::ok)
                .map(|entry| entry.into_path())
                .inspect(|path| println!("cargo:rerun-if-changed={}", path.display())),
        )
        .try_compile("lz4")
        .map_err(|err| Error::new(ErrorKind::Other, err))
}
