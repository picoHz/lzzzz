use lzzzz::{
    lz4f,
    lz4f::{
        compressor::{CompressorBuilder, WriteCompressor},
        decompress_to_vec,
    },
};
use std::io::{Read, Write};

#[test]
fn parallel_compression_decompression() {
    super::run(|state| {
        let data = super::lz4f::generate_data(state, 1024);
        let pref = super::lz4f::generate_preference(state).build();
        let err = |_| (data.clone(), pref);

        let mut comp = Vec::new();
        {
            let mut stream = CompressorBuilder::new(&mut comp)
                .preferences(pref)
                .build::<WriteCompressor<_>>()
                .map_err(err)?;
            stream.write_all(&data).map_err(|_| (data.clone(), pref))?;
        }

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
