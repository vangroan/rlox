use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rlox_gc::{Collector};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("alloc 8Byte", |b| {
        let mut collector = Collector::new();

        b.iter(|| collector.alloc(black_box([0u8; 8])));

        collector.collect()
    });

    c.bench_function("alloc 24Byte", |b| {
        let mut collector = Collector::new();

        b.iter(|| collector.alloc(black_box([0u8; 24])));

        collector.collect()
    });

    c.bench_function("alloc 32Byte", |b| {
        let mut collector = Collector::new();

        b.iter(|| collector.alloc(black_box([0u8; 32])));

        collector.collect()
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
