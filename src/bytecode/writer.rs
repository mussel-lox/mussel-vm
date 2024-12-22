use std::{collections::HashMap, io::Cursor};

use anyhow::{bail, Result};
use byteorder::WriteBytesExt;

use crate::bytecode::{Bytecode, Constant, ConstantIndex, Endianness, OperationCode};

/// A shallow encapsulation of [`Bytecode`].
///
/// Supported data (e.g. [`OperationCode`], `u16`, etc.) can be written into bytecode conveniently,
/// without considering the endianness and how they are turned into `u8` bytes. Internally,
/// [`Cursor`] from the standard library is used and [`Endianness`] is adopted.
///
/// Moreover, constant caching is implemented. If a constant is [`BytecodeWriter::define`]-ed
/// before, it will not be defined again when calling the same function, and its index will be
/// returned directly.
pub struct BytecodeWriter<'a> {
    cursor: Cursor<&'a mut Vec<u8>>,
    constants: &'a mut Vec<Constant>,
    constant_cache: HashMap<Constant, ConstantIndex>,
}

impl<'a> BytecodeWriter<'a> {
    /// Create a BytecodeWriter.
    ///
    /// BytecodeWriter does not own a [`Bytecode`], it just borrows one, in order to reduce
    /// unnecessary moving and improve performance.
    pub fn new(bytecode: &'a mut Bytecode) -> Self {
        Self {
            cursor: Cursor::new(&mut bytecode.code),
            constants: &mut bytecode.constants,
            constant_cache: HashMap::new(),
        }
    }

    /// Define a constant, returning its index.
    ///
    /// Note that constant caching is implemented, which means the index of an existing constant
    /// will be directly returned without writing it twice in the constant section of [`Bytecode`].
    pub fn define(&mut self, constant: Constant) -> Result<ConstantIndex> {
        if let Some(index) = self.constant_cache.get(&constant) {
            return Ok(*index);
        }
        // Define a new constant.
        if self.constants.len() > ConstantIndex::MAX as usize {
            bail!("too many constants");
        }
        let index = self.constants.len() as ConstantIndex;
        self.constants.push(constant.clone());
        self.constant_cache.insert(constant, index);
        Ok(index)
    }
}

/// Helper trait to write bytecode conveniently.
///
/// User does not need to call different methods when emitting different types into [`Bytecode`].
/// Just call `emit(...)` and let the compiler handles it.
pub trait Emit<T> {
    fn emit(&mut self, value: T) -> Result<()>;
}

impl Emit<OperationCode> for BytecodeWriter<'_> {
    fn emit(&mut self, value: OperationCode) -> Result<()> {
        Ok(self.cursor.write_u8(value as u8)?)
    }
}

impl Emit<u8> for BytecodeWriter<'_> {
    fn emit(&mut self, value: u8) -> Result<()> {
        Ok(self.cursor.write_u8(value)?)
    }
}

macro_rules! emit_primitives_impl {
    ($($t: ty), *) => {
        paste::paste! {
            $(
            impl Emit<$t> for BytecodeWriter<'_> {
                fn emit(&mut self, value: $t) -> Result<()> {
                    Ok(self.cursor.[<write_ $t>]::<Endianness>(value)?)
                }
            }
            )*
        }
    };
}

emit_primitives_impl!(u16, i16);
