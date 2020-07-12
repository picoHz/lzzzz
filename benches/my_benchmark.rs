use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lzzzz::lz4;

fn lz4_compress(src: &[u8]) -> Vec<u8> {
    let mut buf = Vec::new();
    lz4::compress_to_vec(src, &mut buf, lz4::ACC_LEVEL_DEFAULT).unwrap();
    buf
}

fn lz4_compress_fast(src: &[u8]) -> Vec<u8> {
    let mut buf = Vec::new();
    lz4::compress_to_vec(src, &mut buf, 10000).unwrap();
    buf
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let data = "En vérité, ne ferait-on pas, pour moins que cela, le Tour du Monde ?";
    c.bench_function("lz4_compress", |b| {
        b.iter(|| lz4_compress(black_box(data.as_bytes())))
    });
    c.bench_function("lz4_compress_fast", |b| {
        b.iter(|| lz4_compress_fast(black_box(data.as_bytes())))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
