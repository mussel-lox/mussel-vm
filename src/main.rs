use anyhow::Result;
use mussel_vm::{
    bytecode,
    bytecode::{CallPosition, Constant, ConstantIndex, LocalOffset, OperationCode},
    vm::VirtualMachine,
};

fn main() -> Result<()> {
    let bytecode = bytecode! {
        const [
            Constant::Number(114.0),
            Constant::Number(514.0),
        ]

        OperationCode::Fun; 15 as CallPosition; 2 as LocalOffset; // var f = add;
        OperationCode::Constant; 0 as ConstantIndex;              // f(114, 514);
        OperationCode::Constant; 1 as ConstantIndex;
        OperationCode::GetLocal; 0 as LocalOffset;
        OperationCode::Invoke;
        OperationCode::Print;
        OperationCode::Return;
                                                                // fun add(a, b) {
        OperationCode::GetLocal; 0 as LocalOffset;              //      return a + b;
        OperationCode::GetLocal; 1 as LocalOffset;
        OperationCode::Add;
        OperationCode::Return;
                                                                // }
    };

    let mut vm = VirtualMachine::new();
    vm.interpret(&bytecode)
}
