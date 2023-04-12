use flame;
use rlox_core::{Chunk, LoxVm, OpCode};
use std::fs::File;

fn run() {
    let mut chunk = Chunk::new();

    let index = chunk.add_constant(1.2);
    chunk.write(OpCode::Constant, 123);
    chunk.write(index, 123);

    let index = chunk.add_constant(3.4);
    chunk.write(OpCode::Constant, 123);
    chunk.write(index, 123);

    chunk.write(OpCode::Add, 123);

    let index = chunk.add_constant(5.6);
    chunk.write(OpCode::Constant, 123);
    chunk.write(index, 123);

    chunk.write(OpCode::Divide, 123);

    chunk.write(OpCode::Negate, 123);

    chunk.write(OpCode::Return, 124);

    println!("{}", chunk.disassemble_to_string().unwrap());

    let mut vm = LoxVm::new();
    let value = vm.interpret(chunk).unwrap();
    println!("Result: {value}");
}

fn main() {
    for _ in 0..1 {
        run();
    }

    let _ = std::fs::create_dir("flame-graphs");
    let mut file = File::create("flame-graphs/scratch.html").expect("creating flame graph report");
    flame::dump_html(&mut file).expect("fumping flame graph html");
}
