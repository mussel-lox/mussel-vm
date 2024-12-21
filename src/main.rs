use anyhow::Result;
use mussel_vm::{
    bytecode,
    bytecode::{Constant, ConstantIndex, LocalOffset, OperationCode},
    vm::VirtualMachine,
};

fn main() -> Result<()> {
    let bytecode = bytecode! {
        const [
            Constant::String("outer".into()),
            Constant::String("inner".into()),
        ]
                                                     // {
        OperationCode::Constant; 0 as ConstantIndex; //     var a = "outer";
                                                     //     {
        OperationCode::Constant; 1 as ConstantIndex; //         var a = "inner";
        OperationCode::GetLocal; 1 as LocalOffset;   //         print a;
        OperationCode::Print;
        OperationCode::Pop;                          //     }
        OperationCode::GetLocal; 0 as LocalOffset;   //     print a;
        OperationCode::Print;
        OperationCode::Pop;                          // }
        OperationCode::Return;
    };

    let mut vm = VirtualMachine::new();
    vm.interpret(&bytecode)
}
