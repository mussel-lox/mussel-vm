use std::{
    fmt::{Display, Formatter},
    hash::Hash,
    ops::Deref
    ,
};

use crate::gc::{FunctionPointer, Reference};

/// The value types of Mussel VM.
///
/// Mussel VM is (at least, originally) designed for the Lox language, thus the Lox types are
/// supported: numbers, strings, booleans, nil and object types.
#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    Nil,
    String(Reference<String>),
    FunctionPointer(Reference<FunctionPointer>),
}

impl Value {
    pub fn as_boolean(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Nil => false,
            _ => true,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(n1), Value::Number(n2)) => (n1 - n2).abs() < f64::EPSILON,
            (Value::Boolean(b1), Value::Boolean(b2)) => b1 == b2,
            (Value::Nil, Value::Nil) => true,
            (Value::String(s1), Value::String(s2)) => {
                if s1 == s2 {
                    return true;
                }
                s1.deref() == s2.deref()
            }
            (Value::FunctionPointer(f1), Value::FunctionPointer(f2)) => {
                if f1 == f2 {
                    return true;
                }
                f1.position == f2.position && f1.arity == f2.arity
            }
            _ => false,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => n.fmt(f),
            Value::Boolean(b) => b.fmt(f),
            Value::Nil => write!(f, "nil"),
            Value::String(s) => s.deref().fmt(f),
            Value::FunctionPointer(fun) => {
                write!(
                    f,
                    "<fun position={:#06X} arity={}>",
                    fun.position, fun.arity
                )
            }
        }
    }
}
