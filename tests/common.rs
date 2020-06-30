use lzzzz::lz4f::*;
use rand::{distributions::Standard, rngs::SmallRng, Rng, SeedableRng};
use std::{i32, u32};

fn generate_data() -> impl Iterator<Item = Vec<u8>> {
    (0..20).map(|n| {
        let rng = SmallRng::seed_from_u64(n as u64);
        rng.sample_iter(Standard).take(16 << n).collect()
    })
}

fn preferences_set() -> impl Iterator<Item = Preferences> {
    vec![
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
            .block_mode(BlockMode::Independent)
            .build(),
        PreferencesBuilder::new()
            .content_checksum(ContentChecksum::Enabled)
            .build(),
        PreferencesBuilder::new().dict_id(u32::MAX).build(),
        PreferencesBuilder::new()
            .block_checksum(BlockChecksum::Enabled)
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
    ]
    .into_iter()
}

pub fn lz4f_test_set() -> impl Iterator<Item = (Vec<u8>, Preferences)> {
    generate_data()
        .map(|data| preferences_set().map(move |prefs| (data.clone(), prefs)))
        .flatten()
}

pub fn lz4_test_set() -> impl Iterator<Item = Vec<u8>> {
    generate_data()
}
