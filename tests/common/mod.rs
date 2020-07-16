#![allow(dead_code)]

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
            .compression_level(CLEVEL_HIGH)
            .build(),
        PreferencesBuilder::new()
            .compression_level(CLEVEL_MAX)
            .build(),
        PreferencesBuilder::new()
            .compression_level(i32::MAX)
            .build(),
        PreferencesBuilder::new()
            .compression_level(i32::MIN)
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

fn compression_acc_set() -> impl Iterator<Item = i32> {
    vec![lz4::ACC_LEVEL_DEFAULT, 0, i32::MIN, i32::MAX].into_iter()
}

pub fn lz4_test_set() -> impl Iterator<Item = (Bytes, i32)> {
    generate_data()
        .map(|data| compression_acc_set().map(move |acc| (data.clone(), acc)))
        .flatten()
}

pub fn lz4_stream_test_set() -> impl Iterator<Item = (Vec<Bytes>, i32)> {
    compression_acc_set().map(|acc| (generate_data().collect(), acc))
}

fn compression_level_set() -> impl Iterator<Item = i32> {
    vec![
        lz4_hc::CLEVEL_DEFAULT,
        lz4_hc::CLEVEL_MIN,
        lz4_hc::CLEVEL_OPT_MIN,
        lz4_hc::CLEVEL_MAX,
        i32::MIN,
        i32::MAX,
    ]
    .into_iter()
}

pub fn lz4_hc_test_set() -> impl Iterator<Item = (Bytes, i32)> {
    generate_data()
        .map(|data| compression_level_set().map(move |level| (data.clone(), level)))
        .flatten()
}

pub fn lz4_hc_stream_test_set() -> impl Iterator<Item = (Vec<Bytes>, i32)> {
    compression_level_set().map(|level| (generate_data().collect(), level))
}
