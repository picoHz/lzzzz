use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lzzzz::lz4_hc;

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

criterion_group!(benches, lz4_hc_benchmark);
criterion_main!(benches);
