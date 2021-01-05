//! Source code chunk.
use crate::{opcode::OpCode, value::Value};
use num_traits::{FromPrimitive, ToPrimitive};
use std::fmt::Write as FmtWrite;

pub struct Chunk {
    constants: Vec<Value>,
    code: Vec<u8>,
    line: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            constants: vec![],
            code: vec![],
            line: vec![],
        }
    }

    /// Adds a constant value to the chunk's constant registry.
    ///
    /// Returns a unique index that can be used to reference the constant.
    ///
    /// # Panic
    ///
    /// Panics if the maximum number of constants has been reached.
    pub fn add_constant<T>(&mut self, constant: T) -> ConstantIndex
    where
        T: Into<Value>,
    {
        assert!(
            self.constants.len() + 1 < std::u16::MAX as usize,
            "Chunk constant vector overflow"
        );
        let index = ConstantIndex(self.constants.len() as u16);
        self.constants.push(constant.into());
        index
    }

    /// Write a single opcode to the chunk's code.
    pub fn push_op(&mut self, opcode: OpCode, line: usize) {
        self.code.push(opcode.to_u8().unwrap());
        self.line.push(line);
    }

    /// Write a single byte to the chunk's code.
    pub fn push_u8(&mut self, instruction: u8, line: usize) {
        self.code.push(instruction);
        self.line.push(line);
    }

    /// Write a single value to the chunk's code.
    pub fn push<T>(&mut self, value: T, line: usize)
    where
        T: PushCode,
    {
        value.push_instruction(self, line);
    }

    /// Write a human readable representation of the bytecode stored in the chunk.
    pub fn disassemble<W>(&self, writer: &mut W) -> Result<(), std::fmt::Error>
    where
        W: FmtWrite,
    {
        writeln!(writer, "=== constants ===")?;
        for (index, constant) in self.constants.iter().enumerate() {
            writeln!(writer, "{:4} {:?}", index, constant)?;
        }

        writeln!(writer, "=== code ===")?;
        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(writer, offset)?;
        }

        Ok(())
    }

    /// Return a string containing a human readable representation of the bytecode stored in the chunk.
    pub fn disassemble_to_string(&self) -> Result<String, std::fmt::Error> {
        let mut s = String::new();
        self.disassemble(&mut s)?;
        Ok(s)
    }

    #[inline]
    fn disassemble_instruction<W>(&self, w: &mut W, offset: usize) -> Result<usize, std::fmt::Error>
    where
        W: FmtWrite,
    {
        write!(w, "{:04x} ", offset)?;

        let instruction = self.code[offset];
        let line = self.line[offset];

        // When an instruction belongs to the same line as a previous one, we
        // print a pipe character instead to make it clear they belong together.
        if offset > 0 && self.line[offset - 1] == line {
            write!(w, "   | ")?;
        } else {
            write!(w, "{:4} ", line)?;
        }

        match OpCode::from_u8(instruction) {
            Some(opcode) => match opcode {
                OpCode::Return => Self::disassemble_instruction_1(w, offset, opcode),
                OpCode::Constant => self.disassemble_constant(w, offset, opcode),
            },
            None => {
                eprintln!("Unknown opcode {:x}", instruction);
                Ok(offset + 1)
            }
        }
    }

    #[inline(always)]
    fn disassemble_instruction_1<W>(w: &mut W, offset: usize, op: OpCode) -> Result<usize, std::fmt::Error>
    where
        W: FmtWrite,
    {
        writeln!(w, "{:?}", op)?;
        Ok(offset + 1)
    }

    #[inline(always)]
    fn disassemble_constant<W>(&self, w: &mut W, offset: usize, op: OpCode) -> Result<usize, std::fmt::Error>
    where
        W: FmtWrite,
    {
        let index = ConstantIndex((self.code[offset + 1] as u16) << 8 | self.code[offset + 2] as u16);
        writeln!(w, "{:?}\t\t{} '{}'", op, index, self.constants[index.0 as usize])?;
        Ok(offset + 3)
    }
}

/// Unique identifier to a constant value stored in a chunk.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ConstantIndex(u16);

impl ConstantIndex {
    #[inline]
    pub fn val(&self) -> u16 {
        self.0
    }
}

impl std::fmt::Display for ConstantIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl PushCode for ConstantIndex {
    fn push_instruction(&self, chunk: &mut Chunk, line: usize) {
        chunk.push_u8((self.0 & 0xF0 >> 8) as u8, line);
        chunk.push_u8((self.0 & 0xF) as u8, line);
    }
}

/// Trait to allow value to be written into a code chunk.
pub trait PushCode {
    fn push_instruction(&self, chunk: &mut Chunk, line: usize);
}

impl PushCode for OpCode {
    fn push_instruction(&self, chunk: &mut Chunk, line: usize) {
        chunk.push_op(*self, line);
    }
}
