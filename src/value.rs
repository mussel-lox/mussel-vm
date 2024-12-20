use std::hash::{Hash, Hasher};
use std::ptr;

/// The value types of Mussel VM.
///
/// Mussel VM is (at least, originally) designed for the Lox language, thus the Lox types are
/// supported: numbers, strings, booleans, nil and object types.
#[derive(Debug)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

impl From<Value> for bool {
    fn from(value: Value) -> Self {
        match value {
            Value::Boolean(b) => b,
            Value::Nil => false,
            _ => true,
        }
    }
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Number(n) => n.to_bits().hash(state),
            Value::String(s) => s.hash(state),
            Value::Boolean(b) => b.hash(state),
            Value::Nil => ptr::null::<()>().hash(state),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(n1), Value::Number(n2)) => (n1 - n2).abs() < f64::EPSILON,
            (Value::String(s1), Value::String(s2)) => s1 == s2,
            (Value::Boolean(b1), Value::Boolean(b2)) => b1 == b2,
            (Value::Nil, Value::Nil) => true,
            _ => false,
        }
    }
}
