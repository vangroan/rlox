//! Core `rlox` compiler and virtual machine.
mod chunk;
mod error;
mod opcode;
mod value;
mod vm;

pub use chunk::Chunk;
pub use error::{LoxError, Result};
pub use opcode::OpCode;
pub use vm::LoxVm;

pub mod prelude {
    pub use crate::chunk::PushCode;
    pub use crate::vm::LoxVm;
}
