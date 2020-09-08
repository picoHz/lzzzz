use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lzzzz::lz4;
use std::i32;

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

criterion_group!(benches, lz4_benchmark);
criterion_main!(benches);
