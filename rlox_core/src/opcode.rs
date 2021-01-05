//! Instruction opcodes.
use num_derive::{FromPrimitive, ToPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ToPrimitive, FromPrimitive)]
pub enum OpCode {
    /// Return from a function.
    Return = 1,
    /// Load a constant value from a chunk.
    /// Followed by an u8 containing an index to the constant.
    /// This is a compact 16-bit operation, allowing for 255 constants. When the
    /// number of constants surpasses this limit, the [`OpCode::ConstantLong`](enum.OpCode.html) opcode is used.
    Constant,
    /// Load a constant value from a chunk.
    /// Followed by 3 instructions (24-bits) containing an index to the constant.
    /// This is a large 32-bit operation, allowing for 2²⁴ constants.
    ConstantLong,
}
