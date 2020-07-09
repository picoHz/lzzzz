#![cfg(feature = "lz4-hc")]

use lzzzz::{lz4, lz4_hc};
use rand::{distributions::Standard, rngs::SmallRng, Rng, SeedableRng};
use rayon::{iter::ParallelBridge, prelude::*};

mod common;
use common::lz4_hc_stream_test_set;

mod compressor {
    use super::*;

    #[test]
    fn default() {
        lz4_hc_stream_test_set()
            .par_bridge()
            .for_each(|(src_set, level)| {
                let mut stream = lz4_hc::Compressor::new().unwrap();
                stream.set_compression_level(level);
                for src in src_set {
                    // TODO
                    if src.len() >= 4194304 {
                        continue;
                    }
                    let mut comp_buf = vec![0; lz4::max_compressed_size(src.len())];
                    let mut decomp_buf = vec![0; src.len()];

                    let len = stream.next(Vec::from(src.as_ref()), &mut comp_buf).unwrap();

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
        lz4_hc_stream_test_set()
            .par_bridge()
            .for_each(|(src_set, level)| {
                let dict = SmallRng::seed_from_u64(0)
                    .sample_iter(Standard)
                    .take(64_000)
                    .collect::<Vec<_>>();
                let mut stream = lz4_hc::Compressor::with_dict(&dict).unwrap();
                stream.set_compression_level(level);
                for src in src_set {
                    // TODO
                    if src.len() >= 4194304 {
                        continue;
                    }
                    let mut comp_buf = vec![0; lz4::max_compressed_size(src.len())];
                    let len = stream.next(Vec::from(src.as_ref()), &mut comp_buf).unwrap();

                    let mut decomp_buf = vec![0; src.len()];
                    lz4::decompress(
                        &comp_buf[..len],
                        &mut decomp_buf,
                        lz4::DecompressionMode::Dictionary { data: &dict },
                    )
                    .unwrap();
                    assert_eq!(src, decomp_buf);
                }
            });
    }

    #[test]
    fn dynamic_adaptation() {
        lz4_hc_stream_test_set()
            .par_bridge()
            .for_each(|(src_set, _)| {
                let mut stream = lz4_hc::Compressor::new().unwrap();
                let mut rng = SmallRng::seed_from_u64(0);
                for src in src_set {
                    // TODO
                    if src.len() >= 4194304 {
                        continue;
                    }
                    let mut comp_buf = vec![0; lz4::max_compressed_size(src.len())];
                    let mut decomp_buf = vec![0; src.len()];

                    stream.set_compression_level(lz4_hc::CompressionLevel::Custom(rng.gen()));
                    stream.set_favor_dec_speed(if rng.gen_bool(0.5) {
                        lz4_hc::FavorDecSpeed::Enabled
                    } else {
                        lz4_hc::FavorDecSpeed::Disabled
                    });

                    let len = stream.next(Vec::from(src.as_ref()), &mut comp_buf).unwrap();

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
}
