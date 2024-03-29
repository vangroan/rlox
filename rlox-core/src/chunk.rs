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
    const CONSTANT_THRESHOLD: usize = u8::MAX as usize;
    const CONSTANT_MASK: usize = 0xFFFFFF; // 2^24
    /// Exclusive maximum number of allowed constants. Limited by max value of 24-bit unsigned integer.
    const CONSTANT_MAX: usize = Self::CONSTANT_MASK + 1;

    pub fn new() -> Self {
        Chunk {
            constants: Vec::with_capacity(Self::CONSTANT_THRESHOLD - 1),
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
    pub fn add_constant(&mut self, constant: impl Into<Value>) -> ConstantIndex {
        assert!(
            self.constants.len() + 1 < Chunk::CONSTANT_MAX,
            "Chunk constant vector overflow"
        );

        let index = if self.constants.len() <= Chunk::CONSTANT_THRESHOLD {
            ConstantIndex::Short(self.constants.len() as u8)
        } else {
            ConstantIndex::Long(self.constants.len() as u32)
        };
        self.constants.push(constant.into());
        index
    }

    /// Explicitly adds a constant value with a 24-bit index.
    ///
    /// Returns a unique index that can be used to reference the constant.
    ///
    /// # Panic
    ///
    /// Panics if the maximum number of constants has been reached.
    pub fn add_constant_long(&mut self, constant: impl Into<Value>) -> ConstantIndex {
        assert!(
            self.constants.len() + 1 < Chunk::CONSTANT_MAX,
            "Chunk constant vector overflow"
        );

        let index = ConstantIndex::Long(self.constants.len() as u32);
        self.constants.push(constant.into());
        index
    }

    #[inline]
    pub fn get_constant_unchecked(&self, index: ConstantIndex) -> &Value {
        unsafe { self.constants.get_unchecked(index.to_usize()) }
    }

    #[inline(always)]
    pub fn get_contant(&self, index: ConstantIndex) -> Option<&Value> {
        self.constants.get(index.to_usize())
    }

    /// Write a single opcode to the chunk's code.
    pub fn write_op(&mut self, opcode: OpCode, line: usize) {
        self.code.push(opcode.to_u8().unwrap());
        self.line.push(line);
    }

    /// Write a single byte to the chunk's code.
    pub fn write_u8(&mut self, instruction: u8, line: usize) {
        self.code.push(instruction);
        self.line.push(line);
    }

    /// Write a single value to the chunk's code.
    pub fn write<T>(&mut self, value: T, line: usize)
    where
        T: EmitCode,
    {
        value.emit_bytecode(self, line);
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
                OpCode::Negate => Self::disassemble_instruction_1(w, offset, opcode),
                OpCode::Add => Self::disassemble_instruction_1(w, offset, opcode),
                OpCode::Subtract => Self::disassemble_instruction_1(w, offset, opcode),
                OpCode::Multiply => Self::disassemble_instruction_1(w, offset, opcode),
                OpCode::Divide => Self::disassemble_instruction_1(w, offset, opcode),
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
            OpCode::Constant => ConstantIndex::Short(self.code[offset + 1]),
            OpCode::ConstantLong => {
                ConstantIndex::from_parts(self.code[offset + 1], self.code[offset + 2], self.code[offset + 3])
            }
            _ => {
                unreachable!("Disassemble constant called with incorrect opcode {:?}", op);
            }
        };
        writeln!(w, "{:?}\t\t{:4} '{}'", op, index, self.constants[index.to_usize()])?;

        match index {
            ConstantIndex::Short(_) => Ok(offset + 2),
            ConstantIndex::Long(_) => Ok(offset + 4),
        }
    }
}

/// Unique identifier to a constant value stored in a chunk.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ConstantIndex {
    /// Index to the first 256 constants.
    Short(u8),
    /// Index to the rest of the constants. Only the 24 lower (rightmost) bits are used, because
    /// the whole 32-bit instruction also contains an 8-bit opcode.
    Long(u32),
}

impl ConstantIndex {
    #[inline(always)]
    pub fn from_u8(value: u8) -> Self {
        ConstantIndex::Short(value)
    }

    /// Create a constant index from a 32-bit integer.
    pub fn from_u32(value: u32) -> Self {
        assert!(value < Chunk::CONSTANT_MAX as u32, "constant index must fit in 24 bits");

        if value <= Chunk::CONSTANT_THRESHOLD as u32 {
            ConstantIndex::Short(value as u8)
        } else {
            ConstantIndex::Long(value)
        }
    }

    /// Assemble a 24-bit index from instruction bytes.
    ///
    /// The parts must be in big-endian order, with the most significant byte first and least significant last.
    ///
    /// The upper 8-bits will be unused.
    #[inline]
    pub fn from_parts(x: u8, y: u8, z: u8) -> Self {
        ConstantIndex::Long((x as u32) << 16 | (y as u32) << 8 | z as u32)
    }

    #[inline]
    pub fn val(&self) -> u32 {
        match self {
            ConstantIndex::Short(value) => *value as u32,
            ConstantIndex::Long(value) => *value,
        }
    }

    #[inline]
    pub fn to_usize(&self) -> usize {
        self.val() as usize
    }

    #[inline]
    pub fn is_u8(&self) -> bool {
        matches!(self, ConstantIndex::Short(_))
    }

    #[inline]
    pub fn is_u24(&self) -> bool {
        matches!(self, ConstantIndex::Long(_))
    }
}

impl std::fmt::Display for ConstantIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConstantIndex::Short(value) => value.fmt(f),
            ConstantIndex::Long(value) => value.fmt(f),
        }
    }
}

impl EmitCode for ConstantIndex {
    fn emit_bytecode(&self, chunk: &mut Chunk, line: usize) {
        match self {
            ConstantIndex::Short(value) => {
                chunk.write_u8(*value, line);
            }
            ConstantIndex::Long(value) => {
                // Big-endian
                chunk.write_u8(((*value >> 16) & 0xFF) as u8, line);
                chunk.write_u8(((*value >> 8) & 0xFF) as u8, line);
                chunk.write_u8((*value & 0xFF) as u8, line);
            }
        }
    }
}

/// Trait to allow value to be written into a code chunk.
pub trait EmitCode {
    fn emit_bytecode(&self, chunk: &mut Chunk, line: usize);
}

impl EmitCode for OpCode {
    fn emit_bytecode(&self, chunk: &mut Chunk, line: usize) {
        chunk.write_op(*self, line);
    }
}

impl EmitCode for u8 {
    fn emit_bytecode(&self, chunk: &mut Chunk, line: usize) {
        chunk.write_u8(*self, line)
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

            if i <= Chunk::CONSTANT_THRESHOLD {
                chunk.write(OpCode::Constant, 123);
                chunk.write(index, 123);

                assert!(index.is_u8());
                assert_eq!(index, ConstantIndex::from_u8(i as u8));
            } else {
                chunk.write(OpCode::ConstantLong, 123);
                chunk.write(index, 123);

                assert!(index.is_u24());
            }
        }

        // println!("{}", chunk.disassemble_to_string().unwrap());
    }
}
