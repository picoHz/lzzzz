use lzzzz::{lz4, lz4_hc};
use rand::{distributions::Standard, rngs::SmallRng, Rng, SeedableRng};
use rayon::{iter::ParallelBridge, prelude::*};
use static_assertions::assert_impl_all;

mod common;
use common::lz4_hc_stream_test_set;

assert_impl_all!(lz4_hc::Compressor: Send);

mod compressor {
    use super::*;

    #[test]
    fn default() {
        lz4_hc_stream_test_set().par_bridge().for_each(|param| {
            let mut comp = lz4_hc::Compressor::new().unwrap();
            let mut decomp = lz4::Decompressor::new().unwrap();

            #[cfg(not(feature = "system-liblz4"))]
            comp.set_compression_level(param.1);

            for src in param.0 {
                let mut comp_buf = vec![0; lz4::max_compressed_size(src.len())];
                let len = comp.next(&src, &mut comp_buf).unwrap();
                assert_eq!(decomp.next(&comp_buf[..len], src.len()).unwrap(), &src);
            }
        });
    }

    #[test]
    fn dictionary() {
        lz4_hc_stream_test_set().par_bridge().for_each(|param| {
            let dict = SmallRng::seed_from_u64(0)
                .sample_iter(Standard)
                .take(64 * 1024)
                .collect::<Vec<_>>();
            let mut comp = lz4_hc::Compressor::with_dict(&dict).unwrap();
            let mut decomp = lz4::Decompressor::with_dict(&dict).unwrap();

            #[cfg(not(feature = "system-liblz4"))]
            comp.set_compression_level(param.1);

            for src in param.0 {
                let mut comp_buf = vec![0; lz4::max_compressed_size(src.len())];
                let len = comp.next(&src, &mut comp_buf).unwrap();
                assert_eq!(decomp.next(&comp_buf[..len], src.len()).unwrap(), &src);
            }
        });
    }

    #[test]
    fn dynamic_adaptation() {
        lz4_hc_stream_test_set()
            .par_bridge()
            .for_each(|(src_set, _)| {
                let mut comp = lz4_hc::Compressor::new().unwrap();
                let mut decomp = lz4::Decompressor::new().unwrap();
                for src in src_set {
                    let mut comp_buf = vec![0; lz4::max_compressed_size(src.len())];

                    #[cfg(not(feature = "system-liblz4"))]
                    {
                        let mut rng = SmallRng::seed_from_u64(0);
                        comp.set_favor_dec_speed(if rng.gen_bool(0.5) {
                            lz4_hc::FavorDecSpeed::Enabled
                        } else {
                            lz4_hc::FavorDecSpeed::Disabled
                        });
                        comp.set_compression_level(rng.gen());
                    }

                    let len = comp.next(&src, &mut comp_buf).unwrap();
                    assert_eq!(decomp.next(&comp_buf[..len], src.len()).unwrap(), &src);
                }
            });
    }
}
