use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rlox_gc::Collector;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("alloc");

    group.throughput(Throughput::Bytes(8));
    group.bench_function(BenchmarkId::from_parameter(8), |b| {
        let mut collector = Collector::new();

        b.iter(|| collector.alloc(black_box([0u8; 8])));

        collector.collect()
    });

    group.throughput(Throughput::Bytes(24));
    group.bench_function(BenchmarkId::from_parameter(24), |b| {
        let mut collector = Collector::new();

        b.iter(|| collector.alloc(black_box([0u8; 24])));

        collector.collect()
    });

    group.throughput(Throughput::Bytes(32));
    group.bench_function(BenchmarkId::from_parameter(32), |b| {
        let mut collector = Collector::new();

        b.iter(|| collector.alloc(black_box([0u8; 32])));

        collector.collect()
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
