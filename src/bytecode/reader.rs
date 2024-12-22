use std::{
    io::{Cursor, Seek, SeekFrom},
    mem,
};

use byteorder::ReadBytesExt;

use crate::bytecode::{Bytecode, Constant, Endianness, OperationCode};

/// A shallow encapsulation of [`Bytecode`].
///
/// For operation codes and operands, just call `fetch()`. The offset, endianness and type
/// conversion is considered internally. For constants, just call `load()`.
pub struct BytecodeReader<'a> {
    cursor: Cursor<&'a Vec<u8>>,
    constants: &'a Vec<Constant>,
}

impl<'a> BytecodeReader<'a> {
    /// Create a BytecodeReader.
    ///
    /// BytecodeReader immutably borrows a [`Bytecode`]. That should be easy to optimize and thus
    /// get performance improvements for Rust compiler.
    pub fn new(bytecode: &'a Bytecode) -> Self {
        Self {
            cursor: Cursor::new(&bytecode.code),
            constants: &bytecode.constants,
        }
    }

    pub fn position(&self) -> usize {
        self.cursor.position() as usize
    }

    /// Load a constant if any, panics if index out of bounds.
    pub fn load(&mut self, index: usize) -> Constant {
        if index >= self.constants.len() {
            panic!("constant index {} out of bounds", index);
        }
        self.constants[index].clone()
    }

    pub fn jump(&mut self, offset: isize) {
        self.cursor.seek_relative(offset as i64).unwrap();
    }

    pub fn seek(&mut self, index: usize) {
        self.cursor.seek(SeekFrom::Start(index as u64)).unwrap();
    }
}

/// Helper trait to read operation codes and operands conveniently.
///
/// User does not need to call different methods when fetching operation codes or operands in
/// different types. Just call `fetch()` (with type annotations usually) and let the compiler
/// handles it.
pub trait Fetch<T> {
    fn fetch(&mut self) -> T;
}

impl Fetch<OperationCode> for BytecodeReader<'_> {
    fn fetch(&mut self) -> OperationCode {
        let candidate = self.cursor.read_u8().unwrap();
        if candidate >= OperationCode::Impossible as u8 {
            panic!("invalid operation code {}", candidate);
        }
        unsafe { mem::transmute(candidate) }
    }
}

impl Fetch<u8> for BytecodeReader<'_> {
    fn fetch(&mut self) -> u8 {
        self.cursor.read_u8().unwrap()
    }
}

macro_rules! fetch_primitives_impl {
    ($($t: ty), *) => {
        paste::paste! {
            $(
            impl Fetch<$t> for BytecodeReader<'_> {
                fn fetch(&mut self) -> $t {
                    self.cursor.[<read_ $t>]::<Endianness>().unwrap()
                }
            }
            )*
        }
    };
}

fetch_primitives_impl!(u16, i16);
