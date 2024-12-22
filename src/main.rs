use anyhow::Result;
use mussel_vm::{
    bytecode,
    bytecode::{CallIndex, Constant, ConstantIndex, LocalOffset, OperationCode},
    vm::VirtualMachine,
};

fn main() -> Result<()> {
    let bytecode = bytecode! {
        const [
            Constant::Number(114.0),
            Constant::Number(514.0),
        ]

        OperationCode::Constant; 0 as ConstantIndex;            // print add(114, 514);
        OperationCode::Constant; 1 as ConstantIndex;
        OperationCode::Call; 12 as CallIndex; 2 as LocalOffset;
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
