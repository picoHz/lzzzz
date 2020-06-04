use cc::Build;
use globwalk;
use std::io::{Error, ErrorKind};

fn main() -> Result<(), Error> {
    let sources: Vec<_> = globwalk::glob("vendor/liblz4/lib/**/*.c")?
        .filter_map(Result::ok)
        .map(|entry| entry.into_path())
        .collect();

    for path in &sources {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    
    Build::new()
        .files(sources)
        .try_compile("lz4")
        .map_err(|err| Error::new(ErrorKind::Other, err))
}
