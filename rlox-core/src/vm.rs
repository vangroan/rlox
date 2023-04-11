//! Virtual machine state.
use crate::chunk::ConstantIndex;
use crate::{
    chunk::Chunk,
    error::{self, LoxError},
    opcode::OpCode,
    value::Value,
};
use num_traits::FromPrimitive;
use rlox_derive::array_init;
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
            stack: array_init!(Value, [Value::Null; LoxVm::STACK_MAX]),
        }
    }

    #[allow(dead_code)]
    #[inline]
    fn peek_mut(&mut self, offset: isize) -> &mut Value {
        // Top cursor points to the element just after the actual top element.
        let index = -1 + self.top as isize + offset;
        assert!(index >= 0, "Stack underflow");
        assert!(index < Self::STACK_MAX as isize, "Stack overflow");

        unsafe { self.stack.get_unchecked_mut(index as usize) }
    }

    #[inline]
    fn push(&mut self, value: Value) {
        #[cfg(feature = "profile")]
        let _ = flame::start_guard("vm push");

        assert!(self.top < Self::STACK_MAX, "Stack overflow");

        // Top index points to just past the top element.
        self.stack[self.top] = value;
        self.top += 1;
    }

    #[inline]
    fn pop(&mut self) -> Value {
        #[cfg(feature = "profile")]
        let _ = flame::start_guard("vm pop");

        assert!(self.top > 0, "Stack underflow");

        self.top -= 1;
        let mut value = Value::Null;
        std::mem::swap(&mut value, &mut self.stack[self.top]);
        value
    }

    #[inline]
    fn try_pop(&mut self) -> Value {
        #[cfg(feature = "profile")]
        let _ = flame::start_guard("vm try_pop");

        if self.top >= 1 {
            self.top -= 1;
        }
        let mut value = Value::Null;
        std::mem::swap(&mut value, &mut self.stack[self.top]);
        value
    }

    pub fn interpret(&mut self, chunk: Chunk) -> error::Result<Value> {
        self.chunk = chunk;
        self.ip = 0;

        if self.chunk.len() > 0 {
            self.run()
        } else {
            Ok(Value::Null)
        }
    }

    #[inline(always)]
    fn get_byte(&mut self) -> u8 {
        let b = self.chunk.get_byte(self.ip);
        self.ip += 1;
        b
    }

    fn get_3bytes(&mut self) -> [u8; 3] {
        let x = self.chunk.get_byte(self.ip);
        let y = self.chunk.get_byte(self.ip + 1);
        let z = self.chunk.get_byte(self.ip + 2);
        self.ip += 3;
        [x, y, z]
    }

    /// Checks whether the instruction pointer is at the end of the chunk.
    fn at_end(&self) -> bool {
        self.ip >= self.chunk.len()
    }

    fn run(&mut self) -> error::Result<Value> {
        #[cfg(feature = "profile")]
        let _ = flame::start_guard("vm run");

        #[cfg(feature = "trace-execution")]
        let mut buf = String::new();

        loop {
            #[cfg(feature = "profile")]
            let _ = flame::start_guard("vm loop");

            #[cfg(feature = "trace-execution")]
            {
                println!("{:?}", &self.stack[0..self.top]);
                self.chunk.disassemble_instruction(&mut buf, self.ip).unwrap();
                print!("{}", buf);
                buf.clear();
            }

            if self.at_end() {
                return Ok(self.try_pop());
            }

            let op = OpCode::from_u8(self.get_byte());

            #[cfg(feature = "profile")]
            let _ = flame::start_guard("vm opcode dispatch");

            match op {
                Some(OpCode::Constant) => {
                    #[cfg(feature = "profile")]
                    let _ = flame::start_guard("opcode Constant");

                    let index = ConstantIndex::from_u8(self.get_byte());
                    let constant = self.chunk.get_contant(index).cloned().unwrap_or(Value::Null);
                    self.push(constant);
                }
                Some(OpCode::ConstantLong) => {
                    #[cfg(feature = "profile")]
                    let _ = flame::start_guard("opcode ConstantLong");

                    let [x, y, z] = self.get_3bytes();
                    let index = ConstantIndex::from_u24_parts(x, y, z);
                    let constant = self.chunk.get_contant(index).cloned().unwrap_or(Value::Null);
                    self.push(constant);
                }
                Some(OpCode::Negate) => {
                    #[cfg(feature = "profile")]
                    let _ = flame::start_guard("opcode Negate");

                    let value = self.pop();
                    arithmetic_op!(self, -value);
                }
                Some(OpCode::Add) => {
                    #[cfg(feature = "profile")]
                    let _ = flame::start_guard("opcode Add");

                    let b = self.pop();
                    let a = self.pop();
                    arithmetic_op!(self, a + b);
                }
                Some(OpCode::Subtract) => {
                    #[cfg(feature = "profile")]
                    let _ = flame::start_guard("opcode Subtract");

                    let b = self.pop();
                    let a = self.pop();
                    arithmetic_op!(self, a - b);
                }
                Some(OpCode::Multiply) => {
                    #[cfg(feature = "profile")]
                    let _ = flame::start_guard("opcode Multiply");

                    let b = self.pop();
                    let a = self.pop();
                    arithmetic_op!(self, a * b);
                }
                Some(OpCode::Divide) => {
                    #[cfg(feature = "profile")]
                    let _ = flame::start_guard("opcode Divide");

                    let b = self.pop();
                    let a = self.pop();
                    arithmetic_op!(self, a / b);
                }
                Some(OpCode::Return) => {
                    #[cfg(feature = "profile")]
                    let _ = flame::start_guard("opcode Return");

                    // println!("Interpret return {}", self.pop());
                    return Ok(self.try_pop());
                }
                _ => {}
            }
        }
    }
}
