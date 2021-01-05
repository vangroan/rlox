use rlox_core::{Chunk, LoxVm, OpCode};

fn main() {
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

    println!("{}", chunk.disassemble_to_string().unwrap());

    let mut vm = LoxVm::new();
    vm.interpret(chunk).unwrap();
}
