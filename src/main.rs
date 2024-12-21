use anyhow::Result;
use mussel_vm::{
    bytecode,
    bytecode::{Constant, ConstantIndex, JumpOffset, OperationCode},
    vm::VirtualMachine,
};

fn main() -> Result<()> {
    let bytecode = bytecode! {
        const [
            Constant::Number(114.0),
            Constant::Number(514.0),
            Constant::Number(1919810.0),
            Constant::String("OK".into()),
            Constant::String("WTF".into()),
        ]

        OperationCode::Constant; 0 as ConstantIndex; // if (114 + 514 < 1919810)
        OperationCode::Constant; 1 as ConstantIndex;
        OperationCode::Add;
        OperationCode::Constant; 2 as ConstantIndex;
        OperationCode::Less;
        OperationCode::JumpIfFalse; 7 as JumpOffset; // {
        OperationCode::Constant; 3 as JumpOffset;    //     print "OK";
        OperationCode::Print;
        OperationCode::Jump; 4 as JumpOffset;        // } else {
        OperationCode::Constant; 4 as JumpOffset;    //     print "WTF";
        OperationCode::Print;
                                                     // }
        OperationCode::Return;
    };

    let mut vm = VirtualMachine::new();
    vm.interpret(&bytecode)
}
