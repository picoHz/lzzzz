use lzzzz::{lz4f, lz4f::StreamCompressor};
use std::io::Read;

#[test]
fn parallel_compression_decompression() {
    super::run(|state| {
        let data = super::lz4f::generate_data(state, 1024);
        let pref = super::lz4f::generate_preference(state).build();
        let err = |_| (data.clone(), pref);

        let mut stream = StreamCompressor::new(data.as_slice(), pref).map_err(err)?;

        let mut comp = Vec::new();
        stream
            .read_to_end(&mut comp)
            .map_err(|_| (data.clone(), pref))?;

        let mut decomp = Vec::new();
        let r = lz4f::decompress_to_vec(&comp, &mut decomp);

        r.map_err(|_| (data.clone(), pref))?;
        if data == decomp {
            Ok(())
        } else {
            Err((data.clone(), pref))
        }
    });
}
