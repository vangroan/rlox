use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rlox_gc::{Collector};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("collect 8Byte * 1000000", |b| {
        let mut collector = Collector::new();
        let mut objs = vec![];
        for _ in 0..1000000 {
            objs.push(collector.alloc([0u8; 8]));
        }

        b.iter(|| collector.collect())
    });

    c.bench_function("collect 24Byte * 1000000", |b| {
        let mut collector = Collector::new();
        let mut objs = vec![];
        for _ in 0..1000000 {
            objs.push(collector.alloc([0u8; 24]));
        }

        b.iter(|| collector.collect())
    });

    c.bench_function("collect 32Byte * 1000000", |b| {
        let mut collector = Collector::new();
        let mut objs = vec![];
        for _ in 0..1000000 {
            objs.push(collector.alloc([0u8; 32]));
        }

        b.iter(|| collector.collect())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
