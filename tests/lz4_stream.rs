#![cfg(feature = "lz4")]

use lzzzz::lz4;
use rand::{distributions::Standard, rngs::SmallRng, Rng, SeedableRng};
use rayon::{iter::ParallelBridge, prelude::*};

mod common;
use common::lz4_stream_test_set;

mod compressor {
    use super::*;

    #[test]
    fn default() {
        lz4_stream_test_set()
            .par_bridge()
            .for_each(|(src_set, mode)| {
                let mut stream = lz4::Compressor::new().unwrap();
                for src in src_set {
                    let mut comp_buf = vec![0; lz4::max_compressed_size(src.len())];
                    let mut decomp_buf = vec![0; src.len()];

                    let len = stream
                        .next(Vec::from(src.as_ref()), &mut comp_buf, mode)
                        .unwrap()
                        .dst_len();

                    lz4::decompress(
                        &comp_buf[..len],
                        &mut decomp_buf,
                        lz4::DecompressionMode::Default,
                    )
                    .unwrap();
                    assert_eq!(decomp_buf, src);
                }
            });
    }

    #[test]
    fn dictionary() {
        lz4_stream_test_set()
            .par_bridge()
            .for_each(|(src_set, mode)| {
                let dict = SmallRng::seed_from_u64(0)
                    .sample_iter(Standard)
                    .take(64_000)
                    .collect::<Vec<_>>();
                let mut stream = lz4::Compressor::with_dict(&dict).unwrap();
                for src in src_set {
                    let mut comp_buf = vec![0; lz4::max_compressed_size(src.len())];
                    let len = stream
                        .next(Vec::from(src.as_ref()), &mut comp_buf, mode)
                        .unwrap()
                        .dst_len();

                    let mut decomp_buf = vec![0; src.len()];
                    lz4::decompress(
                        &comp_buf[..len],
                        &mut decomp_buf,
                        lz4::DecompressionMode::Dictionary {
                            data: (&dict).into(),
                        },
                    )
                    .unwrap();
                    assert_eq!(src, decomp_buf);
                }
            });
    }

    #[test]
    fn pseudo_contiguous_region() {
        let mut comp = lz4::Compressor::new().unwrap();
        let mut region = vec![0; 1000];
        for i in 0..10 {
            for b in region.iter_mut() {
                *b = i as u8;
            }
            let mut comp_buf = Vec::new();
            let region = &region[(i * 100)..][..100];
            comp.next_to_vec(
                #[allow(unsafe_code)]
                unsafe {
                    std::slice::from_raw_parts(region.as_ptr(), region.len())
                },
                &mut comp_buf,
                lz4::CompressionMode::Default,
            )
            .unwrap();

            let mut decomp_buf = vec![0; region.len()];
            lz4::decompress(&comp_buf, &mut decomp_buf, lz4::DecompressionMode::Default).unwrap();
            assert_eq!(decomp_buf.as_slice(), region);
        }
    }
}
