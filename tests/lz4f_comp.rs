use lzzzz::{
    lz4f,
    lz4f::{
        AutoFlush, BlockSize, CompressionLevel, FavorDecSpeed, Preferences, PreferencesBuilder,
    },
};
use rand::{distributions::Standard, rngs::SmallRng, Rng, SeedableRng};
use std::i32;

fn test_compress_to_vec(prefs: &Preferences) {
    for src in generate_data() {
        let mut comp_buf = Vec::new();
        let mut decomp_buf = Vec::new();

        lz4f::compress_to_vec(&src, &mut comp_buf, prefs).unwrap();
        lz4f::decompress_to_vec(&comp_buf, &mut decomp_buf).unwrap();
        assert_eq!(decomp_buf, src);
    }
}

fn generate_data() -> impl Iterator<Item = Vec<u8>> {
    (0..24).map(|n| {
        let rng = SmallRng::seed_from_u64(n as u64);
        rng.sample_iter(Standard).take(2 << n).collect()
    })
}

#[test]
fn compress_to_vec() {
    let prefs = [
        PreferencesBuilder::new().build(),
        PreferencesBuilder::new()
            .block_size(BlockSize::Max64KB)
            .build(),
        PreferencesBuilder::new()
            .block_size(BlockSize::Max256KB)
            .build(),
        PreferencesBuilder::new()
            .block_size(BlockSize::Max1MB)
            .build(),
        PreferencesBuilder::new()
            .block_size(BlockSize::Max4MB)
            .build(),
        PreferencesBuilder::new()
            .compression_level(CompressionLevel::High)
            .build(),
        PreferencesBuilder::new()
            .compression_level(CompressionLevel::Max)
            .build(),
        PreferencesBuilder::new()
            .compression_level(CompressionLevel::Custom(i32::MAX))
            .build(),
        PreferencesBuilder::new()
            .compression_level(CompressionLevel::Custom(i32::MIN))
            .build(),
        PreferencesBuilder::new()
            .favor_dec_speed(FavorDecSpeed::Enabled)
            .build(),
        PreferencesBuilder::new()
            .auto_flush(AutoFlush::Enabled)
            .build(),
    ];

    for p in prefs.iter() {
        test_compress_to_vec(&p);
    }
}
