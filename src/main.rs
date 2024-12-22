use anyhow::Result;
use mussel_vm::{
    bytecode,
    bytecode::{CallIndex, Constant, ConstantIndex, LocalOffset, OperationCode},
    vm::VirtualMachine,
};

fn main() -> Result<()> {
    let bytecode = bytecode! {
        const [
            Constant::String("Hello".into()),
            Constant::String("World".into()),
        ]

        OperationCode::Constant; 0 as ConstantIndex;
        OperationCode::Print;
        OperationCode::Call; 10 as CallIndex; 0 as LocalOffset;
        OperationCode::Pop;
        OperationCode::Return;

        OperationCode::Constant; 1 as ConstantIndex;
        OperationCode::Print;
        OperationCode::Nil;
        OperationCode::Return;
    };

    let mut vm = VirtualMachine::new();
    vm.interpret(&bytecode)
}
