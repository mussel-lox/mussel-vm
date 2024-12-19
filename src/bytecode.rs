use anyhow::{bail, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;
use std::mem;

/// The operation codes.
///
/// Operation codes forms the virtual ISA, which is recognized by the virtual machine (VM). It's
/// a dense, linear sequence of instruction and is good for performance. Tree structures at the
/// source code level (e.g. control flows) are implemented by several kinds of jump instructions.
#[repr(u8)]
pub enum OperationCode {
    Constant,
    Add,
    Subtract,
    Multiply,
    Divide,
    Return,

    /// Guard variant to detect invalid operation codes.
    Impossible,
}

impl TryFrom<u8> for OperationCode {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value >= OperationCode::Impossible as u8 {
            bail!("invalid operation code {}", value);
        }
        Ok(unsafe { mem::transmute(value) })
    }
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
pub enum Constant {
    Number(f64),
    String(String),
}

/// The bytecode.
///
/// Bytecode is the binary representation of a program. As Niklaus Wirth describes, the bytecode
/// is also the combination of data structure and algorithms. More specifically, a [`Bytecode`]
/// of Mussel VM consists of a [`OperationCode`] sequence and some [`Constant`]s.
pub struct Bytecode {
    pub code: Vec<u8>,
    pub constants: Vec<Constant>,
}

macro_rules! extract_primitive_impl {
    ($($operand: ty), *) => {
        impl Bytecode {
            paste::paste! {
                $(
                pub fn [<extract_ $operand>](&self, offset: usize) -> Result<($operand, usize)> {
                    let operand_size = mem::size_of::<$operand>();
                    if self.code.len() - offset <= operand_size {
                        bail!(
                            "operation code {} with insufficient operand size {}",
                            self.code[offset],
                            operand_size,
                        );
                    }
                    let mut cursor = Cursor::new(&self.code[offset + 1 ..]);
                    let operand = cursor.[<read_ $operand>]::<LittleEndian>()?;
                    Ok((operand, operand_size))
                }
                )*
            }
        }
    };
}

extract_primitive_impl!(u16);
