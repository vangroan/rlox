//! Dynamically typed value.
use std::{
    fmt,
    fmt::Debug,
    ops::{Add, Div, Mul, Neg, Sub},
};

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Float(f64),
    Err,
}

impl Value {
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Float(v) => Some(*v),
            _ => None,
        }
    }
}

impl Default for Value {
    #[inline]
    fn default() -> Self {
        Value::Null
    }
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
            Value::Err => write!(f, "error"),
        }
    }
}

impl Neg for Value {
    type Output = Self;
    fn neg(self) -> Self {
        match self {
            Value::Float(v) => Value::Float(-v),
            _ => Value::Err,
        }
    }
}

impl Add for Value {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
            _ => Value::Err,
        }
    }
}

impl Sub for Value {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
            _ => Value::Err,
        }
    }
}

impl Mul for Value {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Value::Float(a), Value::Float(b)) => Value::Float(a * b),
            _ => Value::Err,
        }
    }
}

impl Div for Value {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Value::Float(a), Value::Float(b)) => Value::Float(a / b),
            _ => Value::Err,
        }
    }
}
