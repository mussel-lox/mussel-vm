use anyhow::Result;
use mussel_vm::bytecode;
use mussel_vm::bytecode::{Constant, ConstantIndex, OperationCode};
use mussel_vm::vm::VirtualMachine;

fn main() -> Result<()> {
    let bytecode = bytecode! {
        const [
            Constant::Number(114.0),
            Constant::Number(514.0),
        ]

        OperationCode::Constant; 0 as ConstantIndex;
        OperationCode::Constant; 1 as ConstantIndex;
        OperationCode::Add;
        OperationCode::Return;
    };

    let mut vm = VirtualMachine::new();
    vm.interpret(&bytecode)
}
