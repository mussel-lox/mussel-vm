use mussel_vm::{
    bytecode,
    bytecode::{CallPosition, Constant, ConstantIndex, GlobalIndex, LocalOffset, OperationCode},
    vm::VirtualMachine,
};

fn main() {
    let bytecode = bytecode! {
        //fun hello() {
        //     var world = 1;
        //     fun theworld() {
        //         world = world - 1;
        //         return world;
        //     }
        //     world = world + 114514;
        //     return theworld;
        // }
        //
        // var standpower = hello();
        // print standpower();
        // print standpower();
        //
        // ================Output================
        // 114514
        // 114513
        //
        // ===============Bytecode===============
        //
        // const [
        //     00 <1>
        //     01 <114514>
        // ]
        //
        // main:
        //     00 CALL      16 0
        //     04 SETGLOBAL 0
        //     06 POP
        //     07 GETGLOBAL 0
        //     09 INVOKE
        //     10 PRINT
        //     11 GETGLOBAL 0
        //     13 INVOKE
        //     14 PRINT
        //     15 RETURN
        //
        // hello:
        //     16 CONSTANT 00
        //     19 CLOSURE  35 0    ; new opcode, create a new Closure object based on CallPosition and arity.
        //     23 CAPTURE  0       ; new opcode, box local variable and bind it to the closure object at stack top.
        //     25 GETLOCAL 0
        //     27 CONSTANT 01
        //     30 ADD
        //     31 SETLOCAL 0
        //     33 POP
        //     34 RETURN
        //
        //
        // hello$theworld:
        //     35 GETUPVALUE 0     ; new opcode, get an upvalue from the closure object itself.
        //     37 CONSTANT   00
        //     40 SUBTRACT
        //     41 SETUPVALUE 0     ; new opcode, update an upvalue by a Value on stack top.
        //     43 POP
        //     44 GETUPVALUE 0
        //     46 RETURN

        const [
            Constant::Number(1.0),
            Constant::Number(114514.0),
        ]

        OperationCode::Call; 16 as CallPosition; 0 as LocalOffset;
        OperationCode::SetGlobal; 0 as GlobalIndex;
        OperationCode::Pop;
        OperationCode::GetGlobal; 0 as GlobalIndex;
        OperationCode::Invoke;
        OperationCode::Print;
        OperationCode::GetGlobal; 0 as GlobalIndex;
        OperationCode::Invoke;
        OperationCode::Print;
        OperationCode::Return;

        OperationCode::Constant; 0 as ConstantIndex;
        OperationCode::Closure; 35 as CallPosition; 0 as LocalOffset;
        OperationCode::Capture; 0 as LocalOffset;
        OperationCode::GetLocal; 0 as LocalOffset;
        OperationCode::Constant; 1 as ConstantIndex;
        OperationCode::Add;
        OperationCode::SetLocal; 0 as LocalOffset;
        OperationCode::Pop;
        OperationCode::Return;

        OperationCode::GetUpvalue; 0 as LocalOffset;
        OperationCode::Constant; 0 as ConstantIndex;
        OperationCode::Subtract;
        OperationCode::SetUpvalue; 0 as LocalOffset;
        OperationCode::Pop;
        OperationCode::GetUpvalue; 0 as LocalOffset;
        OperationCode::Return;
    };

    let mut vm = VirtualMachine::new();
    vm.interpret(&bytecode);
}
