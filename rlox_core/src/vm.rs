//! Virtual machine state.
use crate::chunk::ConstantIndex;
use crate::{
    chunk::Chunk,
    error::{self, LoxError},
    opcode::OpCode,
    value::Value,
};
use num_traits::FromPrimitive;
use rlox_derive::array;
#[cfg(feature = "trace-execution")]
use std::fmt::Write as FmtWrite;

/// Helper for handling type checking on expressions that result in a `Value` containing a numerical type.
#[doc(hidden)]
macro_rules! arithmetic_op {
    ($vm:ident, $op:expr) => {
        match $op {
            value @ Value::Float(_) => $vm.push(value),
            Value::Err => return Err(LoxError::TypeError),
            _ => unreachable!("Operator not implemented"),
        }
    };
}

pub struct LoxVm {
    chunk: Chunk,
    ip: usize,
    /// Index to element just past the top element in the value stack.
    top: usize,
    stack: [Value; LoxVm::STACK_MAX],
}

impl LoxVm {
    const STACK_MAX: usize = 256;

    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            ip: 0,
            top: 0,
            stack: array!(Value, [Value::Null; LoxVm::STACK_MAX]),
        }
    }

    fn push(&mut self, value: Value) {
        assert!(self.top < Self::STACK_MAX, "Stack overflow");

        // Top index points to just past the top element.
        self.stack[self.top] = value;
        self.top += 1;
    }

    fn pop(&mut self) -> Value {
        assert!(self.top > 0, "Stack underflow");

        self.top -= 1;
        let mut value = Value::Null;
        std::mem::swap(&mut value, &mut self.stack[self.top]);
        value
    }

    pub fn interpret(&mut self, chunk: Chunk) -> error::Result<()> {
        self.chunk = chunk;
        self.ip = 0;

        if self.chunk.len() > 0 {
            self.run()
        } else {
            Ok(())
        }
    }

    #[inline(always)]
    fn get_byte(&mut self) -> u8 {
        let b = self.chunk.get_byte(self.ip);
        self.ip += 1;
        b
    }

    fn run(&mut self) -> error::Result<()> {
        #[cfg(feature = "trace-execution")]
        let mut buf = String::new();

        loop {
            #[cfg(feature = "trace-execution")]
            println!("{:?}", &self.stack[0..self.top]);
            #[cfg(feature = "trace-execution")]
            self.chunk.disassemble_instruction(&mut buf, self.ip).unwrap();
            #[cfg(feature = "trace-execution")]
            print!("{}", buf);
            #[cfg(feature = "trace-execution")]
            buf.clear();

            let op = OpCode::from_u8(self.get_byte());

            match op {
                Some(OpCode::Constant) => {
                    let index = ConstantIndex::from_u8(self.get_byte());
                    let constant = self.chunk.get_constant_unchecked(index).clone();
                    self.push(constant);
                }
                Some(OpCode::Negate) => {
                    let value = self.pop();
                    arithmetic_op!(self, -value);
                }
                Some(OpCode::Add) => {
                    let b = self.pop();
                    let a = self.pop();
                    arithmetic_op!(self, a + b);
                }
                Some(OpCode::Subtract) => {
                    let b = self.pop();
                    let a = self.pop();
                    arithmetic_op!(self, a - b);
                }
                Some(OpCode::Multiply) => {
                    let b = self.pop();
                    let a = self.pop();
                    arithmetic_op!(self, a * b);
                }
                Some(OpCode::Divide) => {
                    let b = self.pop();
                    let a = self.pop();
                    arithmetic_op!(self, a / b);
                }
                Some(OpCode::Return) => {
                    println!("Interpret return {}", self.pop());
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}
