use anyhow::Result;
use mussel_vm::bytecode::{Bytecode, BytecodeWriter, Constant, EmitBytecodeExt, OperationCode};
use mussel_vm::vm::VirtualMachine;

macro_rules! bytecode {
    (const [$($constant: expr), * $(,)?] $($code: expr); * $(;)?) => {
        {
            let mut bytecode = Bytecode::new();
            let mut writer = BytecodeWriter::new(&mut bytecode);
            $( writer.define($constant)?; )*
            $( writer.emit($code)?; )*
            bytecode
        }
    };
}

fn main() -> Result<()> {
    let bytecode = bytecode! {
        const [
            Constant::Number(114.0),
            Constant::Number(514.0),
        ]

        OperationCode::Constant; 0u16;
        OperationCode::Constant; 1u16;
        OperationCode::Add;
        OperationCode::Return;
    };
    let mut vm = VirtualMachine::new();
    vm.interpret(&bytecode)
}
