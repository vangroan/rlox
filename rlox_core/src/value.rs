//! Dynamically typed value.
use std::fmt;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Float(f64),
}

impl Into<Value> for f64 {
    fn into(self) -> Value {
        Value::Float(self)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Float(value) => fmt::Display::fmt(value, f),
        }
    }
}
