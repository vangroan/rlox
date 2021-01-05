//! Core `rlox` compiler and virtual machine.
mod chunk;
mod opcode;
mod value;

pub use chunk::Chunk;
pub use opcode::OpCode;

pub mod prelude {
    pub use crate::chunk::PushCode;
}
