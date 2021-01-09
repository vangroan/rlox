use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use rlox_gc::Collector;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("alloc");

    for count in &[1000, 10000, 1000000] {
        group.throughput(Throughput::Bytes(count * 8));
        group.bench_function(format!("8/{}", count), |b| {
            let mut collector = Collector::new();
            let mut objs = vec![];
            for _ in 0..1000000 {
                objs.push(collector.alloc(black_box([0u8; 8])));
            }

            b.iter(|| collector.collect())
        });

        group.throughput(Throughput::Bytes(count * 24));
        group.bench_function(format!("24/{}", count), |b| {
            let mut collector = Collector::new();
            let mut objs = vec![];
            for _ in 0..1000000 {
                objs.push(collector.alloc(black_box([0u8; 24])));
            }

            b.iter(|| collector.collect())
        });

        group.throughput(Throughput::Bytes(count * 32));
        group.bench_function(format!("32/{}", count), |b| {
            let mut collector = Collector::new();
            let mut objs = vec![];
            for _ in 0..1000000 {
                objs.push(collector.alloc(black_box([0u8; 32])));
            }

            b.iter(|| collector.collect())
        });
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
