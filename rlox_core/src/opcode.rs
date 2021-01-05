//! Instruction opcodes.
use num_derive::{FromPrimitive, ToPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ToPrimitive, FromPrimitive)]
pub enum OpCode {
    /// Return from a function.
    Return = 1,
    /// Load a constant value from a chunk.
    Constant,
}
