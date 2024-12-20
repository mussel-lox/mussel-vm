use anyhow::Result;
use mussel_vm::bytecode;
use mussel_vm::bytecode::{Constant, ConstantIndex, OperationCode};
use mussel_vm::value::Value;
use mussel_vm::vm::VirtualMachine;

fn main() -> Result<()> {
    println!("sizeof Value: {}", size_of::<Value>());
    let bytecode = bytecode! {
        const [
            Constant::String("Hello, ".to_string()),
            Constant::String("World!".to_string()),
        ]

        OperationCode::Constant; 0 as ConstantIndex;
        OperationCode::Constant; 1 as ConstantIndex;
        OperationCode::Add;
        OperationCode::Return;
    };

    let mut vm = VirtualMachine::new();
    vm.interpret(&bytecode)
}
