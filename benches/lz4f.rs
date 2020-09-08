use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lzzzz::lz4f;
use std::{
    i32,
    io::{Read, Write},
};

fn lz4f_compress(prefs: &lz4f::Preferences, data: &[u8]) {
    let mut buf = [0u8; 4096];
    lz4f::compress(data, &mut buf, prefs).unwrap();
}

fn lz4f_decompress(data: &[u8]) {
    let mut buf = Vec::new();
    lz4f::decompress_to_vec(data, &mut buf).unwrap();
}

fn lz4f_write_compressor(n: usize, prefs: lz4f::Preferences, data: &[u8]) {
    let mut buf = Vec::new();
    let mut w = lz4f::WriteCompressor::new(&mut buf, prefs).unwrap();
    for _ in 0..n {
        w.write_all(data).unwrap();
    }
}

fn lz4f_bufread_compressor(n: usize, prefs: lz4f::Preferences, data: &[u8]) {
    let mut buf = Vec::new();
    let mut r = lz4f::BufReadCompressor::new(data, prefs).unwrap();
    for _ in 0..n {
        r.read_to_end(&mut buf).unwrap();
    }
}

fn lz4f_benchmark(c: &mut Criterion) {
    let data = include_bytes!("lorem-ipsum.txt");

    c.bench_function("lz4f::compress (Default)", |b| {
        let prefs = lz4f::PreferencesBuilder::new().build();
        b.iter(|| lz4f_compress(&prefs, black_box(data)))
    });

    c.bench_function("lz4f::compress (compression_level: i32::MAX)", |b| {
        let prefs = lz4f::PreferencesBuilder::new()
            .compression_level(i32::MAX)
            .build();
        b.iter(|| lz4f_compress(&prefs, black_box(data)))
    });

    c.bench_function("lz4f::compress (compression_level: i32::MIN)", |b| {
        let prefs = lz4f::PreferencesBuilder::new()
            .compression_level(i32::MIN)
            .build();
        b.iter(|| lz4f_compress(&prefs, black_box(data)))
    });

    c.bench_function("lz4f::WriteCompressor (Default)", |b| {
        let prefs = lz4f::PreferencesBuilder::new().build();
        b.iter(|| lz4f_write_compressor(32, prefs, black_box(data)))
    });

    c.bench_function("lz4f::BufReadCompressor (Default)", |b| {
        let prefs = lz4f::PreferencesBuilder::new().build();
        b.iter(|| lz4f_bufread_compressor(32, prefs, black_box(data)))
    });

    let mut compressed = Vec::new();
    lz4f::compress_to_vec(data, &mut compressed, &Default::default()).unwrap();

    c.bench_function("lz4f::decompress", |b| {
        b.iter(|| lz4f_decompress(black_box(&compressed)))
    });
}

criterion_group!(benches, lz4f_benchmark);
criterion_main!(benches);
