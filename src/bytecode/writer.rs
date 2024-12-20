use crate::bytecode::{Bytecode, Constant, OperationCode, ENDIANNESS};
use anyhow::{bail, Result};
use byteorder::WriteBytesExt;
use std::collections::HashMap;
use std::io::Cursor;

/// A shallow encapsulation of [`Bytecode`].
///
/// Writing bytecode, including operation codes, operands and constants is easy using this struct.
/// Constants are cached: if a constant has appeared before, its index will be returned directly.
pub struct BytecodeWriter<'a> {
    cursor: Cursor<&'a mut Vec<u8>>,
    constants: &'a mut Vec<Constant>,
    constant_cache: HashMap<Constant, usize>,
}

impl<'a> BytecodeWriter<'a> {
    pub fn new(bytecode: &'a mut Bytecode) -> Self {
        Self {
            cursor: Cursor::new(&mut bytecode.code),
            constants: &mut bytecode.constants,
            constant_cache: HashMap::new(),
        }
    }

    pub fn define(&mut self, constant: Constant) -> Result<usize> {
        if let Some(index) = self.constant_cache.get(&constant) {
            return Ok(*index);
        }
        // Define a new constant.
        if self.constants.len() > u16::MAX as usize {
            bail!("too many constants");
        }
        let index = self.constants.len();
        self.constants.push(constant.clone());
        self.constant_cache.insert(constant, index);
        Ok(index)
    }
}

/// Helper trait to write bytecode conveniently.
pub trait EmitBytecodeExt<T> {
    fn emit(&mut self, value: T) -> Result<()>;
}

impl EmitBytecodeExt<u8> for BytecodeWriter<'_> {
    fn emit(&mut self, value: u8) -> Result<()> {
        self.cursor.write_u8(value)?;
        Ok(())
    }
}

impl EmitBytecodeExt<OperationCode> for BytecodeWriter<'_> {
    fn emit(&mut self, value: OperationCode) -> Result<()> {
        self.cursor.write_u8(value as u8)?;
        Ok(())
    }
}

macro_rules! emit_primitives_impl {
    ($($t: ty), *) => {
        paste::paste! {
            $(
            impl EmitBytecodeExt<$t> for BytecodeWriter<'_> {
                fn emit(&mut self, value: $t) -> Result<()> {
                    Ok(self.cursor.[<write_ $t>]::<ENDIANNESS>(value)?)
                }
            }
            )*
        }
    };
}

emit_primitives_impl!(u16);
