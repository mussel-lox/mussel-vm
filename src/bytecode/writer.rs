use std::io::Cursor;

use byteorder::WriteBytesExt;

use crate::bytecode::{Bytecode, Constant, ConstantIndex, Endianness, OperationCode};

/// A shallow encapsulation of [`Bytecode`].
///
/// Supported data (e.g. [`OperationCode`], `u16`, etc.) can be written into bytecode conveniently, without
/// considering the endianness and how they are turned into `u8` bytes. Internally, [`Cursor`] from the standard
/// library is used and [`Endianness`] is adopted.
pub struct BytecodeWriter<'a> {
    cursor: Cursor<&'a mut Vec<u8>>,
    constants: &'a mut Vec<Constant>,
}

impl<'a> BytecodeWriter<'a> {
    /// Create a BytecodeWriter.
    ///
    /// BytecodeWriter does not own a [`Bytecode`], it just borrows one, in order to reduce unnecessary moving and
    /// improve performance.
    pub fn new(bytecode: &'a mut Bytecode) -> Self {
        Self {
            cursor: Cursor::new(&mut bytecode.code),
            constants: &mut bytecode.constants,
        }
    }

    /// Define a constant, returning its [`ConstantIndex`].
    pub fn define(&mut self, constant: Constant) -> ConstantIndex {
        // Define a new constant.
        if self.constants.len() > ConstantIndex::MAX as usize {
            panic!("too many constants");
        }
        let index = self.constants.len() as ConstantIndex;
        self.constants.push(constant);
        index
    }
}

/// Helper trait to write bytecode conveniently.
///
/// User does not need to call different methods when emitting different types into [`Bytecode`]. Just call
/// `emit(...)` and let the compiler handles it.
pub trait Emit<T> {
    fn emit(&mut self, value: T);
}

impl Emit<OperationCode> for BytecodeWriter<'_> {
    fn emit(&mut self, value: OperationCode) {
        self.cursor.write_u8(value as u8).unwrap();
    }
}

impl Emit<u8> for BytecodeWriter<'_> {
    fn emit(&mut self, value: u8) {
        self.cursor.write_u8(value).unwrap();
    }
}

macro_rules! emit_primitives_impl {
    ($($t: ty), *) => {
        paste::paste! {
            $(
            impl Emit<$t> for BytecodeWriter<'_> {
                fn emit(&mut self, value: $t)  {
                    self.cursor.[<write_ $t>]::<Endianness>(value).unwrap();
                }
            }
            )*
        }
    };
}

emit_primitives_impl!(u16, i16);
