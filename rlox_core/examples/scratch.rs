use rlox_core::{Chunk, OpCode};

fn main() {
    let mut chunk = Chunk::new();

    let index = chunk.add_constant(2.1);
    chunk.push(OpCode::Constant, 123);
    chunk.push(index, 123);

    chunk.push(OpCode::Return, 123);

    println!("{}", chunk.disassemble_to_string().unwrap());
}
