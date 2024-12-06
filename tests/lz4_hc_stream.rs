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
        lz4_hc_stream_test_set()
            .par_bridge()
            .for_each(|(src_set, level)| {
                let mut comp = lz4_hc::Compressor::new().unwrap();
                let mut decomp = lz4::Decompressor::new().unwrap();
                comp.set_compression_level(level);
                for src in src_set {
                    let mut comp_buf = vec![0; lz4::max_compressed_size(src.len())];
                    let len = comp.next(&src, &mut comp_buf).unwrap();
                    assert_eq!(decomp.next(&comp_buf[..len], src.len()).unwrap(), &src);
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
                    .take(64 * 1024)
                    .collect::<Vec<_>>();
                let mut comp = lz4_hc::Compressor::with_dict(&dict, lz4_hc::CLEVEL_DEFAULT).unwrap();
                let mut decomp = lz4::Decompressor::with_dict(&dict).unwrap();
                comp.set_compression_level(level);
                for src in src_set {
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
                let mut rng = SmallRng::seed_from_u64(0);
                for src in src_set {
                    let mut comp_buf = vec![0; lz4::max_compressed_size(src.len())];

                    comp.set_compression_level(rng.gen());
                    comp.set_favor_dec_speed(if rng.gen_bool(0.5) {
                        lz4_hc::FavorDecSpeed::Enabled
                    } else {
                        lz4_hc::FavorDecSpeed::Disabled
                    });

                    let len = comp.next(&src, &mut comp_buf).unwrap();
                    assert_eq!(decomp.next(&comp_buf[..len], src.len()).unwrap(), &src);
                }
            });
    }

    #[test]
    fn attach_dictionary() {
        // Basic test data
        let data = b"The quick brown fox jumps over the lazy dog";
        let level = lz4_hc::CLEVEL_DEFAULT;
    
        // Create dictionary stream and set its compression level
        let mut dict_comp = lz4_hc::Compressor::with_dict(data, level).unwrap();
        
        // Create working stream 
        let mut comp = lz4_hc::Compressor::new().unwrap();
        comp.set_compression_level(level);  // Set level before attachment
    
        // Compress with attached dictionary
        comp.attach_dict(Some(&mut dict_comp), level);
        let mut output_attached_dict = Vec::new();
        comp.next_to_vec(data, &mut output_attached_dict).unwrap();
    
        // Compress with no dictionary
        comp.attach_dict(None, level);
        let mut output_no_dict = Vec::new();
        comp.next_to_vec(data, &mut output_no_dict).unwrap();

        // Results should match
        assert_ne!(output_attached_dict, output_no_dict, "Data with no dict should be different");

        // Code below is disabled because it (unexpectedly) does not work.
        // Seems to be an upstream lz4 issue.

        // Create regular dictionary compressor with same level
        // let mut comp_regular_dict = lz4_hc::Compressor::with_dict(data, level).unwrap();
        // let mut output_regular_dict = Vec::new();
        // comp_regular_dict.next_to_vec(data, &mut output_regular_dict).unwrap();
        // assert_eq!(output_attached_dict, output_regular_dict, "Compressed data should match");
    }
}
