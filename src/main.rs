use anyhow::Result;
use mussel_vm::bytecode::{Bytecode, BytecodeWriter, Constant, EmitBytecodeExt, OperationCode};
use mussel_vm::vm::VirtualMachine;

fn main() -> Result<()> {
    let mut bytecode = Bytecode::new();
    {
        let mut writer = BytecodeWriter::new(&mut bytecode);
        writer.emit(OperationCode::Constant)?;
        writer.emit(0u16)?;
        writer.emit(OperationCode::Constant)?;
        writer.emit(1u16)?;
        writer.emit(OperationCode::Add)?;
        writer.emit(OperationCode::Return)?;

        writer.define(Constant::Number(114.0))?;
        writer.define(Constant::Number(514.0))?;
    }
    let mut vm = VirtualMachine::new();
    vm.interpret(&bytecode)
}
