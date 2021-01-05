//! Errors
use std::{error::Error, fmt};

#[derive(Debug)]
pub enum LoxError {
    /// Error during script compilation.
    Compile,
    /// Error during script execution.
    Runtime,
}

impl Error for LoxError {}

impl fmt::Display for LoxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoxError::Compile => write!(f, "compilation error"),
            LoxError::Runtime => write!(f, "runtime error"),
        }
    }
}

pub type Result<T> = std::result::Result<T, LoxError>;
