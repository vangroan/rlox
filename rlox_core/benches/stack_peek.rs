use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rlox_core::{Chunk, LoxVm, OpCode};

fn create_chunk() -> Chunk {
    let mut chunk = Chunk::new();

    let index = chunk.add_constant(1.2);
    chunk.push(OpCode::Constant, 123);
    chunk.push(index, 123);

    let index = chunk.add_constant(3.4);
    chunk.push(OpCode::Constant, 123);
    chunk.push(index, 123);

    chunk.push(OpCode::Add, 123);

    let index = chunk.add_constant(5.6);
    chunk.push(OpCode::Constant, 123);
    chunk.push(index, 123);

    chunk.push(OpCode::Divide, 123);

    chunk.push(OpCode::Negate, 123);

    chunk.push(OpCode::Return, 123);

    chunk
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut vm = LoxVm::new();

    c.bench_function("negate", |b| {
        b.iter(|| {
            let _ = black_box(20);

            vm.interpret(create_chunk())
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
