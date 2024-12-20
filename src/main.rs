use anyhow::Result;
use mussel_vm::bytecode;
use mussel_vm::bytecode::{Constant, ConstantIndex, GlobalStateIndex, OperationCode};
use mussel_vm::vm::VirtualMachine;

fn main() -> Result<()> {
    let bytecode = bytecode! {
        const [
            Constant::String("beignets".into()),
            Constant::String("cafe au lait".into()),
            Constant::String("beignets with ".into()),
        ]

        OperationCode::Constant; 0 as ConstantIndex;
        OperationCode::SetGlobal; 0 as GlobalStateIndex;
        OperationCode::Pop;
        OperationCode::Constant; 1 as ConstantIndex;
        OperationCode::SetGlobal; 1 as GlobalStateIndex;
        OperationCode::Pop;
        OperationCode::Constant; 2 as ConstantIndex;
        OperationCode::GetGlobal; 1 as GlobalStateIndex;
        OperationCode::Add;
        OperationCode::SetGlobal; 0 as GlobalStateIndex;
        OperationCode::Pop;
        OperationCode::GetGlobal; 0 as GlobalStateIndex;
        OperationCode::Print;
        OperationCode::Return;
    };

    let mut vm = VirtualMachine::new();
    vm.interpret(&bytecode)
}
