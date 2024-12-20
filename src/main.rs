use anyhow::Result;
use mussel_vm::bytecode;
use mussel_vm::bytecode::{Constant, ConstantIndex, OperationCode};
use mussel_vm::vm::VirtualMachine;

fn main() -> Result<()> {
    let bytecode = bytecode! {
        const [
            Constant::Number(5.0),
            Constant::Number(4.0),
            Constant::Number(3.0),
            Constant::Number(2.0),
        ]

        OperationCode::Constant; 0 as ConstantIndex;
        OperationCode::Constant; 1 as ConstantIndex;
        OperationCode::Subtract;
        OperationCode::Constant; 2 as ConstantIndex;
        OperationCode::Constant; 3 as ConstantIndex;
        OperationCode::Multiply;
        OperationCode::Greater;
        OperationCode::Nil;
        OperationCode::Not;
        OperationCode::Equal;
        OperationCode::Not;
        OperationCode::Return;
    };

    let mut vm = VirtualMachine::new();
    vm.interpret(&bytecode)
}
