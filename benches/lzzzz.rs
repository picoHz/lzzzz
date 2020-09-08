use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lzzzz::{lz4, lz4_hc, lz4f};
use std::{
    i32,
    io::{Read, Write},
};

fn lz4_compress(level: i32, data: &[u8]) {
    let mut buf = [0u8; 4096];
    lz4::compress(data, &mut buf, level).unwrap();
}

fn lz4_decompress(orig_len: usize, data: &[u8]) {
    let mut buf = vec![0u8; orig_len];
    lz4::decompress(data, &mut buf).unwrap();
}

fn lz4_compress_streaming(n: usize, level: i32, data: &[u8]) {
    let mut buf = [0u8; 4096];
    let mut comp = lz4::Compressor::new().unwrap();
    for _ in 0..n {
        comp.next(data, &mut buf, level).unwrap();
    }
}

fn lz4_benchmark(c: &mut Criterion) {
    let data = include_bytes!("lorem-ipsum.txt");

    c.bench_function("lz4::compress (ACC_LEVEL_DEFAULT)", |b| {
        b.iter(|| lz4_compress(lz4::ACC_LEVEL_DEFAULT, black_box(data)))
    });

    c.bench_function("lz4::compress (i32::MAX)", |b| {
        b.iter(|| lz4_compress(i32::MAX, black_box(data)))
    });

    c.bench_function("lz4::Compressor (ACC_LEVEL_DEFAULT)", |b| {
        b.iter(|| lz4_compress_streaming(32, lz4::ACC_LEVEL_DEFAULT, black_box(data)))
    });

    c.bench_function("lz4::Compressor (i32::MAX)", |b| {
        b.iter(|| lz4_compress_streaming(32, i32::MAX, black_box(data)))
    });

    let mut compressed = Vec::new();
    lz4::compress_to_vec(data, &mut compressed, lz4::ACC_LEVEL_DEFAULT).unwrap();

    c.bench_function("lz4::decompress", |b| {
        b.iter(|| lz4_decompress(data.len(), black_box(&compressed)))
    });
}

criterion_group!(lz4_benches, lz4_benchmark);

fn lz4_hc_compress(level: i32, data: &[u8]) {
    let mut buf = [0u8; 4096];
    lz4_hc::compress(data, &mut buf, level).unwrap();
}

fn lz4_hc_compress_streaming(n: usize, level: i32, data: &[u8]) {
    let mut buf = [0u8; 4096];
    let mut comp = lz4_hc::Compressor::new().unwrap();
    comp.set_compression_level(level);
    for _ in 0..n {
        comp.next(data, &mut buf).unwrap();
    }
}

fn lz4_hc_benchmark(c: &mut Criterion) {
    let data = include_bytes!("lorem-ipsum.txt");

    c.bench_function("lz4_hc::compress (CLEVEL_DEFAULT)", |b| {
        b.iter(|| lz4_hc_compress(lz4_hc::CLEVEL_DEFAULT, black_box(data)))
    });

    c.bench_function("lz4_hc::compress (CLEVEL_MIN)", |b| {
        b.iter(|| lz4_hc_compress(lz4_hc::CLEVEL_MIN, black_box(data)))
    });

    c.bench_function("lz4_hc::compress (CLEVEL_MAX)", |b| {
        b.iter(|| lz4_hc_compress(lz4_hc::CLEVEL_MAX, black_box(data)))
    });

    c.bench_function("lz4_hc::Compressor (CLEVEL_DEFAULT)", |b| {
        b.iter(|| lz4_hc_compress_streaming(32, lz4_hc::CLEVEL_DEFAULT, black_box(data)))
    });

    c.bench_function("lz4_hc::Compressor (CLEVEL_MIN)", |b| {
        b.iter(|| lz4_hc_compress_streaming(32, lz4_hc::CLEVEL_MIN, black_box(data)))
    });

    c.bench_function("lz4_hc::Compressor (CLEVEL_MAX)", |b| {
        b.iter(|| lz4_hc_compress_streaming(32, lz4_hc::CLEVEL_MAX, black_box(data)))
    });
}

criterion_group!(lz4_hc_benches, lz4_hc_benchmark);

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

criterion_group!(lz4f_benches, lz4f_benchmark);
criterion_main!(lz4_benches, lz4_hc_benches, lz4f_benches);
