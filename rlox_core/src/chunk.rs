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
    /// Threshold where the number of constants changes from using 8-bit indices to 24-bit indices.
    const CONSTANT_THRESHOLD: usize = 256;
    /// Exclusive maximum number of allowed constants. Limited by max value of 24-bit unsigned integer.
    const CONSTANT_MAX: usize = 16_777_216; // 2^24

    pub fn new() -> Self {
        Chunk {
            constants: Vec::with_capacity(Self::CONSTANT_THRESHOLD),
            code: vec![],
            line: vec![],
        }
    }

    /// Retrieve a byte instruction from the chunk code.
    ///
    /// # Panics
    ///
    /// Panics when the given offset is out of bounds.
    #[inline(always)]
    pub fn get_byte(&self, offset: usize) -> u8 {
        self.code[offset]
    }

    /// Returns the number of instructions in the chunk code.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.code.len()
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
            self.constants.len() + 1 < Chunk::CONSTANT_MAX,
            "Chunk constant vector overflow"
        );

        // Comparison is exclusive, as 256 rolls over to the 9th bit.
        let index = if self.constants.len() < Chunk::CONSTANT_THRESHOLD {
            ConstantIndex::U8(self.constants.len() as u8)
        } else {
            ConstantIndex::U24(self.constants.len() as u32)
        };
        self.constants.push(constant.into());
        index
    }

    #[inline]
    pub fn get_constant_unchecked(&self, index: ConstantIndex) -> &Value {
        unsafe { self.constants.get_unchecked(index.to_usize()) }
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
    #[doc(hidden)]
    pub fn disassemble_instruction<W>(&self, w: &mut W, offset: usize) -> Result<usize, std::fmt::Error>
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
                OpCode::NoOp => Self::disassemble_instruction_1(w, offset, opcode),
                OpCode::Return => Self::disassemble_instruction_1(w, offset, opcode),
                OpCode::Constant | OpCode::ConstantLong => self.disassemble_constant(w, offset, opcode),
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
        let index = match op {
            OpCode::Constant => ConstantIndex::U8(self.code[offset + 1]),
            OpCode::ConstantLong => {
                ConstantIndex::from_u24_parts(self.code[offset + 1], self.code[offset + 2], self.code[offset + 3])
            }
            _ => {
                unreachable!("Disassemble constant called with incorrect opcode {:?}", op);
            }
        };
        writeln!(w, "{:?}\t\t{:4} '{}'", op, index, self.constants[index.to_usize()])?;

        match index {
            ConstantIndex::U8(_) => Ok(offset + 2),
            ConstantIndex::U24(_) => Ok(offset + 4),
        }
    }
}

/// Unique identifier to a constant value stored in a chunk.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ConstantIndex {
    /// Index to the first 256 constants.
    U8(u8),
    /// Index to the rest of the constants. Only the 24 lower (rightmost) bits are used, because
    /// the whole 32-bit instruction also contains an 8-bit opcode.
    U24(u32),
}

impl ConstantIndex {
    #[inline(always)]
    pub fn from_u8(value: u8) -> Self {
        ConstantIndex::U8(value)
    }

    /// Assemble an index from instruction bytes.
    ///
    /// The parts must be in big-endian order, with the most significant byte first and least significant last.
    #[inline]
    pub fn from_u24_parts(x: u8, y: u8, z: u8) -> Self {
        ConstantIndex::U24((x as u32) << 16 | (y as u32) << 8 | z as u32)
    }

    #[inline]
    pub fn val(&self) -> u32 {
        match self {
            ConstantIndex::U8(value) => *value as u32,
            ConstantIndex::U24(value) => *value,
        }
    }

    #[inline]
    pub fn to_usize(&self) -> usize {
        self.val() as usize
    }

    #[inline]
    pub fn is_u8(&self) -> bool {
        matches!(self, ConstantIndex::U8(_))
    }

    #[inline]
    pub fn is_u24(&self) -> bool {
        matches!(self, ConstantIndex::U24(_))
    }
}

impl std::fmt::Display for ConstantIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConstantIndex::U8(value) => value.fmt(f),
            ConstantIndex::U24(value) => value.fmt(f),
        }
    }
}

impl PushCode for ConstantIndex {
    fn push_instruction(&self, chunk: &mut Chunk, line: usize) {
        match self {
            ConstantIndex::U8(value) => {
                chunk.push_u8(*value, line);
            }
            ConstantIndex::U24(value) => {
                // Big-endian
                chunk.push_u8(((*value >> 16) & 0xFF) as u8, line);
                chunk.push_u8(((*value >> 8) & 0xFF) as u8, line);
                chunk.push_u8((*value & 0xFF) as u8, line);
            }
        }
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_constant_index() {
        let mut chunk = Chunk::new();

        for i in 0..1024 {
            let index = chunk.add_constant(i as f64);

            if i < Chunk::CONSTANT_THRESHOLD {
                chunk.push(OpCode::Constant, 123);
                chunk.push(index, 123);

                assert!(index.is_u8());
                assert_eq!(index, ConstantIndex::from_u8(i as u8));
            } else {
                chunk.push(OpCode::ConstantLong, 123);
                chunk.push(index, 123);

                assert!(index.is_u24());
            }
        }

        // println!("{}", chunk.disassemble_to_string().unwrap());
    }
}
