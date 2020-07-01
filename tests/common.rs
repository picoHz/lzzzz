use bytes::Bytes;
use lazy_static::lazy_static;
use lzzzz::{lz4, lz4_hc, lz4f::*};
use rand::{distributions::Standard, rngs::SmallRng, Rng, SeedableRng};
use std::{i32, u32};

lazy_static! {
    static ref DATA_SET: Vec<Bytes> = {
        (0..20)
            .map(|n| {
                let rng = SmallRng::seed_from_u64(n as u64);
                rng.sample_iter(Standard).take(16 << n).collect()
            })
            .collect()
    };
}

fn generate_data() -> impl Iterator<Item = Bytes> {
    DATA_SET.clone().into_iter()
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

pub fn lz4f_test_set() -> impl Iterator<Item = (Bytes, Preferences)> {
    generate_data()
        .map(|data| preferences_set().map(move |prefs| (data.clone(), prefs)))
        .flatten()
}

fn compression_mode_set() -> impl Iterator<Item = lz4::CompressionMode> {
    vec![
        lz4::CompressionMode::Default,
        lz4::CompressionMode::Acceleration { factor: 0 },
        lz4::CompressionMode::Acceleration { factor: i32::MIN },
        /* TODO
         * lz4::CompressionMode::Acceleration { factor: i32::MAX }, */
    ]
    .into_iter()
}

pub fn lz4_test_set() -> impl Iterator<Item = (Bytes, lz4::CompressionMode)> {
    generate_data()
        .map(|data| compression_mode_set().map(move |mode| (data.clone(), mode)))
        .flatten()
}

fn compression_level_set() -> impl Iterator<Item = lz4_hc::CompressionLevel> {
    vec![
        lz4_hc::CompressionLevel::Default,
        lz4_hc::CompressionLevel::Min,
        lz4_hc::CompressionLevel::OptMin,
        lz4_hc::CompressionLevel::Max,
        lz4_hc::CompressionLevel::Custom(i32::MIN),
        lz4_hc::CompressionLevel::Custom(i32::MAX),
    ]
    .into_iter()
}

pub fn lz4_hc_test_set() -> impl Iterator<Item = (Bytes, lz4_hc::CompressionLevel)> {
    generate_data()
        .map(|data| compression_level_set().map(move |level| (data.clone(), level)))
        .flatten()
}
