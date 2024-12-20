use crate::bytecode::{Bytecode, Constant, OperationCode, ENDIANNESS};
use anyhow::{bail, Result};
use byteorder::ReadBytesExt;
use std::io::Cursor;
use std::mem;

/// A shallow encapsulation of [`Bytecode`].
///
/// Reading bytecode, including operation codes, operands and constants is easy using this struct.
pub struct BytecodeReader<'a> {
    cursor: Cursor<&'a Vec<u8>>,
    constants: &'a Vec<Constant>,
}

impl<'a> BytecodeReader<'a> {
    pub fn new(bytecode: &'a Bytecode) -> Self {
        Self {
            cursor: Cursor::new(&bytecode.code),
            constants: &bytecode.constants,
        }
    }

    pub fn load(&mut self, index: usize) -> Result<Constant> {
        if index >= self.constants.len() {
            bail!("constant index {} out of bounds", index);
        }
        Ok(self.constants[index].clone())
    }
}

pub trait FetchBytecodeExt<T> {
    fn fetch(&mut self) -> Result<T>;
}

impl FetchBytecodeExt<OperationCode> for BytecodeReader<'_> {
    fn fetch(&mut self) -> Result<OperationCode> {
        let candidate = self.cursor.read_u8()?;
        if candidate >= OperationCode::Impossible as u8 {
            bail!("invalid operation code {}", candidate);
        }
        Ok(unsafe { mem::transmute(candidate) })
    }
}

impl FetchBytecodeExt<u8> for BytecodeReader<'_> {
    fn fetch(&mut self) -> Result<u8> {
        Ok(self.cursor.read_u8()?)
    }
}

macro_rules! fetch_primitives_impl {
    ($($t: ty), *) => {
        paste::paste! {
            $(
            impl FetchBytecodeExt<$t> for BytecodeReader<'_> {
                fn fetch(&mut self) -> Result<$t> {
                    Ok(self.cursor.[<read_ $t>]::<ENDIANNESS>()?)
                }
            }
            )*
        }
    };
}

fetch_primitives_impl!(u16);
