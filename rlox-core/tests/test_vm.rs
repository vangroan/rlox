use rlox_core::{Chunk, LoxVm, OpCode, Value};

#[test]
fn test_arithmetic() {
    let mut chunk = Chunk::new();

    // 1 + 2 * 3
    let const_0 = chunk.add_constant(Value::Float(3.0));
    chunk.push(OpCode::Constant, 0);
    chunk.push(const_0, 0);

    let const_1 = chunk.add_constant(Value::Float(2.0));
    chunk.push(OpCode::Constant, 0);
    chunk.push(const_1, 0);

    chunk.push(OpCode::Multiply, 0);

    // test 24-bit constant index
    let const_2 = chunk.add_constant_long(Value::Float(1.0));
    assert!(const_2.is_u24());
    chunk.push(OpCode::ConstantLong, 0);
    chunk.push(const_2, 0);

    chunk.push(OpCode::Add, 0);

    let mut vm = LoxVm::new();

    let value = vm.interpret(chunk).expect("interpret failed");
    println!("{value:?}");
    assert_eq!(value.as_f64(), Some(7.0));
}
