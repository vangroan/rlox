//! Core `rlox` compiler and virtual machine.
mod chunk;
mod error;
mod opcode;
mod value;
mod vm;

pub use self::chunk::{Chunk, ConstantIndex};
pub use self::error::{LoxError, Result};
pub use self::opcode::OpCode;
pub use self::value::Value;
pub use self::vm::LoxVm;

pub mod prelude {
    pub use super::chunk::EmitCode;
    pub use super::vm::LoxVm;
}
