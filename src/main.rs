use anyhow::Result;
use byteorder::{LittleEndian, WriteBytesExt};
use mussel_vm::bytecode::{Bytecode, Constant, OperationCode};
use mussel_vm::vm::VirtualMachine;
use std::io::Cursor;

fn main() -> Result<()> {
    let mut vm = VirtualMachine::new();
    let mut code = Cursor::new(Vec::<u8>::new());
    code.write_u8(OperationCode::Constant as u8)?;
    code.write_u16::<LittleEndian>(0)?;
    code.write_u8(OperationCode::Constant as u8)?;
    code.write_u16::<LittleEndian>(1)?;
    code.write_u8(OperationCode::Add as u8)?;
    code.write_u8(OperationCode::Return as u8)?;
    let bytecode = Bytecode {
        code: code.into_inner(),
        constants: vec![Constant::Number(114.0), Constant::Number(514.0)],
    };
    vm.interpret(&bytecode)?;
    Ok(())
}
