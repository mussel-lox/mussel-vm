use byteorder::LittleEndian;
use std::hash::{Hash, Hasher};

mod reader;
mod writer;

pub use reader::*;
pub use writer::*;

/// The endianness of bytecode. Used in [`BytecodeReader`] and [`BytecodeWriter`].
pub type ENDIANNESS = LittleEndian;

/// The type of constant index in a [`Bytecode`]. Defined using typedef to deal with possible changes in
/// the future.
pub type ConstantIndex = u16;

/// The operation codes.
///
/// Operation codes forms the virtual ISA, which is recognized by the virtual machine (VM). It's
/// a dense, linear sequence of instruction and is good for performance. Tree structures at the
/// source code level (e.g. control flows) are implemented by several kinds of jump instructions.
#[repr(u8)]
pub enum OperationCode {
    /* Value operations */
    /// Load a constant into the VM stack, with its index stored as `u16` following the operation
    /// code.
    Constant,
    Nil,
    True,
    False,

    /* Unary operations */
    Negate,
    Not,

    /* Binary operations */
    Add,
    Subtract,
    Multiply,
    Divide,
    Return,

    /* Relational operations */
    Equal,
    Greater,
    Less,

    /// Guard variant to detect invalid operation codes.
    Impossible,
}

/// The constants stored in a [`Bytecode`].
///
/// The Mussel VM recognizes some value types (numbers, strings, booleans, object type and nil).
/// However, not all of them can be stored in [`Bytecode`] as constants. For object types,
/// dynamic memory allocation is needed, which is a runtime feature; for boolean and nil
/// literals, there are corresponding operation codes designed to push a value into the VM stack.
///
/// For now, only numbers (internally `f64`) and strings (internally [`String`]) are considered
/// as constants.
#[derive(Debug, Clone)]
pub enum Constant {
    Number(f64),
    String(String),
}

impl Hash for Constant {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Constant::Number(n) => n.to_bits().hash(state),
            Constant::String(s) => s.hash(state),
        }
    }
}

impl PartialEq for Constant {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Constant::Number(n1), Constant::Number(n2)) => (n1 - n2).abs() < f64::EPSILON,
            (Constant::String(s1), Constant::String(s2)) => s1 == s2,
            _ => false,
        }
    }
}

impl Eq for Constant {}

/// The bytecode.
///
/// Bytecode is the binary representation of a program. As Niklaus Wirth describes, the bytecode
/// is also the combination of data structure and algorithms. More specifically, a [`Bytecode`]
/// of Mussel VM consists of a [`OperationCode`] sequence and some [`Constant`]s.
pub struct Bytecode {
    pub code: Vec<u8>,
    pub constants: Vec<Constant>,
}

impl Bytecode {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
        }
    }
}
