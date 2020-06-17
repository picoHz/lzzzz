use lzzzz::{
    lz4f,
    lz4f::{
        co::{BufReadCompressor, ReadCompressor, WriteCompressor},
        decompress_to_vec, CompressorBuilder,
    },
};
use std::io::{BufReader, Read, Write};

#[test]
fn write_compressor() {
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

#[test]
fn read_compressor() {
    super::run(|state| {
        let data = super::lz4f::generate_data(state, 1024);
        let pref = super::lz4f::generate_preference(state).build();
        let err = |_| (data.clone(), pref);

        let mut comp = Vec::new();
        {
            let mut stream = CompressorBuilder::new(data.as_slice())
                .preferences(pref)
                .build::<ReadCompressor<_>>()
                .map_err(err)?;
            stream
                .read_to_end(&mut comp)
                .map_err(|_| (data.clone(), pref))?;
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

fn bufread_compressor() {
    super::run(|state| {
        let data = super::lz4f::generate_data(state, 1024);
        let pref = super::lz4f::generate_preference(state).build();
        let err = |_| (data.clone(), pref);

        let mut comp = Vec::new();
        {
            let mut stream = CompressorBuilder::new(BufReader::new(data.as_slice()))
                .preferences(pref)
                .build::<BufReadCompressor<_>>()
                .map_err(err)?;
            stream
                .read_to_end(&mut comp)
                .map_err(|_| (data.clone(), pref))?;
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
