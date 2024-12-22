use std::{
    io::{Cursor, Seek, SeekFrom},
    mem,
};

use anyhow::{bail, Result};
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

    /// Load a constant if any, reports an [`anyhow::Error`] if index out of bounds.
    pub fn load(&mut self, index: usize) -> Result<Constant> {
        if index >= self.constants.len() {
            bail!("constant index {} out of bounds", index);
        }
        Ok(self.constants[index].clone())
    }

    pub fn jump(&mut self, offset: isize) -> Result<()> {
        Ok(self.cursor.seek_relative(offset as i64)?)
    }

    pub fn seek(&mut self, index: usize) -> Result<()> {
        self.cursor.seek(SeekFrom::Start(index as u64))?;
        Ok(())
    }
}

/// Helper trait to read operation codes and operands conveniently.
///
/// User does not need to call different methods when fetching operation codes or operands in
/// different types. Just call `fetch()` (with type annotations usually) and let the compiler
/// handles it.
pub trait Fetch<T> {
    fn fetch(&mut self) -> Result<T>;
}

impl Fetch<OperationCode> for BytecodeReader<'_> {
    fn fetch(&mut self) -> Result<OperationCode> {
        let candidate = self.cursor.read_u8()?;
        if candidate >= OperationCode::Impossible as u8 {
            bail!("invalid operation code {}", candidate);
        }
        Ok(unsafe { mem::transmute(candidate) })
    }
}

impl Fetch<u8> for BytecodeReader<'_> {
    fn fetch(&mut self) -> Result<u8> {
        Ok(self.cursor.read_u8()?)
    }
}

macro_rules! fetch_primitives_impl {
    ($($t: ty), *) => {
        paste::paste! {
            $(
            impl Fetch<$t> for BytecodeReader<'_> {
                fn fetch(&mut self) -> Result<$t> {
                    Ok(self.cursor.[<read_ $t>]::<Endianness>()?)
                }
            }
            )*
        }
    };
}

fetch_primitives_impl!(u16, i16);
