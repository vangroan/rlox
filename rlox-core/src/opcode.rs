//! Instruction opcodes.
use num_derive::{FromPrimitive, ToPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ToPrimitive, FromPrimitive)]
pub enum OpCode {
    /// Only used by the VM to advance the instruction pointer.
    ///
    /// In the future this will be used to convert u8 to `OpCode` without having to unwrap from `Option`. Unwrapping
    /// introduces additional overhead in the VM loop.
    NoOp = 0,
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
    /// *Arithmetic* Unary negation. Example `-2.1`.
    /// Replaces stack top with a negated value.
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
}
