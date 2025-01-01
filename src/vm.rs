use std::{mem, ops::Deref};

use crate::{
    bytecode::{
        Bytecode, BytecodeReader, CallPosition, Constant, ConstantIndex, Fetch, GlobalIndex,
        JumpOffset, LocalOffset, OperationCode,
    },
    gc::{Allocate, Closure, FunctionPointer, GarbageCollector, Reference},
    stack::Stack,
    value::Value,
};

pub const GLOBALS_CAPACITY: usize = GlobalIndex::MAX as usize + 1;
pub const LOCALS_CAPACITY: usize = LocalOffset::MAX as usize + 1;

struct CallFrame {
    position: CallPosition,
    frame: LocalOffset,
    closure: Option<Reference<Closure>>,
}

/// The Mussel VM.
///
/// A virtual machine stores program states and executes bytecode instructions. As a stack machine, Mussel VM
/// maintains a stack data structure, and stores local variable and does expression evaluation on it.
pub struct VirtualMachine {
    globals: Vec<Value>,
    stack: Stack<Value, LOCALS_CAPACITY>,
    gc: GarbageCollector,
    frame: LocalOffset,
    closure: Option<Reference<Closure>>,
    callstack: Vec<CallFrame>,
}

impl VirtualMachine {
    /// Create a virtual machine.
    pub fn new() -> Self {
        Self {
            globals: vec![Value::Nil; GLOBALS_CAPACITY],
            stack: Stack::new(),
            gc: GarbageCollector::new(),
            frame: 0,
            closure: None,
            callstack: Vec::new(),
        }
    }

    /// Reset the program states, as if the VM is just created and ready to execute bytecode.
    ///
    /// Note that GC is not reset here, it's up to itself to collect garbage.
    pub fn reset(&mut self) {
        self.globals.fill(Value::Nil);
        self.stack.clear();
        self.frame = 0;
        self.callstack.clear();
    }

    /// Execute the bytecode.
    ///
    /// Note that the VM is not reset here, since there may be some needs to execute a piece of bytecode on some
    /// existing program states.
    pub fn interpret(&mut self, bytecode: &Bytecode) {
        let mut reader = BytecodeReader::new(bytecode);
        macro_rules! arithmetic {
            ($operator: tt as $variant: ident) => {{
                // SAFETY: Arithmetic operations can only be applied to numbers, so if there's an operand of a
                // certain reference type, the VM will instantly panic, leaving the GC behavior unimportant.
                let right = self.stack.pop();
                let left = self.stack.pop();
                match (left, right) {
                    (Value::Number(left), Value::Number(right)) => {
                        let result = Value::$variant(left $operator right);
                        self.stack.push(result);
                    }
                    _ => panic!(
                        "arithmetic operator `{}` can only be applied to numbers",
                        stringify!($operator),
                    ),
                }
            }};
        }

        loop {
            let opcode = reader.fetch();
            match opcode {
                OperationCode::Constant => {
                    let index: ConstantIndex = reader.fetch();
                    match reader.load(index as usize) {
                        Constant::Number(n) => self.stack.push(Value::Number(n)),
                        Constant::String(s) => {
                            let allocation = self.gc.allocate(s);
                            self.stack.push(Value::String(allocation));
                        }
                    }
                }
                OperationCode::Nil => self.stack.push(Value::Nil),
                OperationCode::True => self.stack.push(Value::Boolean(true)),
                OperationCode::False => self.stack.push(Value::Boolean(false)),
                OperationCode::Fun => {
                    let position: CallPosition = reader.fetch();
                    let arity: LocalOffset = reader.fetch();
                    let fun = self.gc.allocate(FunctionPointer { position, arity });
                    self.stack.push(Value::FunctionPointer(fun));
                }

                // SAFETY: Negate operation can only be applied to numbers, so if there's an operand of a certain
                // reference type, the VM will instantly panic, leaving the GC behavior unimportant.
                OperationCode::Negate => match self.stack.pop() {
                    Value::Number(n) => self.stack.push(Value::Number(-n)),
                    _ => panic!("negate operator `-` can only be applied to numbers"),
                },

                // SAFETY: Logical not operation can be applied to all kinds of types, including the reference types.
                // However, it does not do dereferencing, so the operand can be GC-ed.
                OperationCode::Not => {
                    let value = self.stack.pop().as_boolean();
                    self.stack.push(Value::Boolean(!value));
                }

                OperationCode::Add => {
                    // SAFETY: Add operation can be applied to numbers or strings, and the latter is a reference type.
                    // We'll have to keep the reference values on stack before evaluation since we cannot know when
                    // the GC will execute.
                    let right = self.stack.peek(0);
                    let left = self.stack.peek(1);
                    match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            let sum = Value::Number(left + right);
                            self.stack.pop();
                            self.stack.pop();
                            self.stack.push(sum);
                        }
                        (Value::String(left), Value::String(right)) => {
                            let concat = self.gc.allocate(format!("{}{}", **left, **right));
                            self.stack.pop();
                            self.stack.pop();
                            self.stack.push(Value::String(concat));
                        }
                        _ => panic!("add operator `+` can only be applied to numbers or strings"),
                    }
                }
                OperationCode::Subtract => arithmetic!(- as Number),
                OperationCode::Multiply => arithmetic!(* as Number),
                OperationCode::Divide => arithmetic!(/ as Number),

                OperationCode::Equal => {
                    // SAFETY: Equal operation can be applied to each kind of values, and there's reference types.
                    // Besides, the overloaded [`PartialEq`] operator actually does do dereferencing, so we'll have
                    // to keep the reference values on stack before evaluation since we cannot know when the GC will
                    // execute.
                    let right = self.stack.peek(0);
                    let left = self.stack.peek(1);
                    let equal = Value::Boolean(left == right);
                    self.stack.pop();
                    self.stack.pop();
                    self.stack.push(equal);
                }
                OperationCode::Greater => arithmetic!(> as Boolean),
                OperationCode::Less => arithmetic!(< as Boolean),

                OperationCode::GetGlobal => {
                    let index: GlobalIndex = reader.fetch();
                    let variable = &self.globals[index as usize];
                    let value = if let Value::Upvalue(u) = variable {
                        u.deref().clone()
                    } else {
                        variable.clone()
                    };
                    self.stack.push(value);
                }
                OperationCode::SetGlobal => {
                    let index: GlobalIndex = reader.fetch();
                    let value = self.stack.top().clone();
                    let target = &mut self.globals[index as usize];
                    if let Value::Upvalue(u) = target {
                        **u = value;
                    } else {
                        *target = value;
                    }
                }

                OperationCode::GetLocal => {
                    let offset: LocalOffset = reader.fetch();
                    let variable = &self.stack[(self.frame + offset) as usize];
                    let value = if let Value::Upvalue(u) = variable {
                        u.deref().clone()
                    } else {
                        variable.clone()
                    };
                    self.stack.push(value);
                }
                OperationCode::SetLocal => {
                    let offset: LocalOffset = reader.fetch();
                    let value = self.stack.top().clone();
                    let target = &mut self.stack[(self.frame + offset) as usize];
                    if let Value::Upvalue(u) = target {
                        **u = value
                    } else {
                        *target = value
                    }
                }

                // No SAFETY here because the Pop operation means to pop a value out of stack directly.
                OperationCode::Pop => drop(self.stack.pop()),

                OperationCode::Closure => {
                    let position: CallPosition = reader.fetch();
                    let arity: LocalOffset = reader.fetch();
                    let closure = self.gc.allocate(Closure {
                        position,
                        arity,
                        upvalues: Vec::new(),
                    });
                    self.stack.push(Value::Closure(closure));
                }
                OperationCode::Capture => {
                    let offset: LocalOffset = reader.fetch();
                    let value = self.stack[(self.frame + offset) as usize].clone();
                    let mut closure = match self.stack.top() {
                        Value::Closure(closure) => *closure,
                        _ => panic!("trying to capture value without closure at the stack top"),
                    };

                    // The only place that creates an upvalue. There will never be a second-order upvalue.
                    if let Value::Upvalue(upvalue) = value {
                        closure.upvalues.push(upvalue);
                    } else {
                        let upvalue = self.gc.allocate(value);
                        self.stack[(self.frame + offset) as usize] = Value::Upvalue(upvalue);
                        closure.upvalues.push(upvalue);
                    }
                }
                OperationCode::GetUpvalue => {
                    let offset: LocalOffset = reader.fetch();
                    let closure = match self.closure {
                        Some(closure) => closure,
                        None => panic!("trying to get upvalue outside a closure"),
                    };
                    let value = closure.upvalues[offset as usize].deref().clone();
                    self.stack.push(value);
                }
                OperationCode::SetUpvalue => {
                    let offset: LocalOffset = reader.fetch();
                    let closure = match self.closure {
                        Some(closure) => closure,
                        None => panic!("trying to set upvalue outside a closure"),
                    };
                    let mut upvalue = closure.upvalues[offset as usize];
                    let value = self.stack.top().clone();
                    *upvalue = value;
                }

                OperationCode::JumpIfFalse => {
                    let offset: JumpOffset = reader.fetch();
                    let condition: bool = self.stack.top().as_boolean();
                    if condition == false {
                        reader.jump(offset as isize);
                    }
                }
                OperationCode::Jump => {
                    let offset: JumpOffset = reader.fetch();
                    reader.jump(offset as isize);
                }
                OperationCode::Call => {
                    let position: CallPosition = reader.fetch();
                    let frame_offset: LocalOffset = reader.fetch();
                    let last_frame = CallFrame {
                        position: reader.position() as CallPosition,
                        frame: self.frame,
                        closure: mem::replace(&mut self.closure, None),
                    };
                    self.callstack.push(last_frame);
                    self.frame = self.stack.len() as LocalOffset - frame_offset;
                    reader.seek(position as usize);
                }
                OperationCode::Invoke => match self.stack.top() {
                    Value::FunctionPointer(f) => {
                        // SAFETY: We get the important part of the function pointer out first, and pops it out of
                        // the stack. It can be GC-ed since we have already known where to call.
                        let position = f.position;
                        let frame_offset = f.arity;
                        self.stack.pop();

                        let last_frame = CallFrame {
                            position: reader.position() as CallPosition,
                            frame: self.frame,
                            closure: mem::replace(&mut self.closure, None),
                        };
                        self.callstack.push(last_frame);
                        self.frame = self.stack.len() as LocalOffset - frame_offset;
                        reader.seek(position as usize);
                    }
                    Value::Closure(c) => {
                        // SAFETY: We get the important part of the function pointer out first, and pops it out of
                        // the stack. It can be GC-ed since we have already known where to call.
                        let position = c.position;
                        let frame_offset = c.arity;
                        let closure = *c;
                        self.stack.pop();

                        let last_frame = CallFrame {
                            position: reader.position() as CallPosition,
                            frame: self.frame,
                            closure: mem::replace(&mut self.closure, None),
                        };
                        self.callstack.push(last_frame);
                        self.frame = self.stack.len() as LocalOffset - frame_offset;
                        self.closure = Some(closure);
                        reader.seek(position as usize);
                    }
                    _ => panic!("object is not callable"),
                },
                OperationCode::Return => {
                    if let Some(last_frame) = self.callstack.pop() {
                        // SAFETY: We don't actually pop the top element out of stack, which may cause GC bugs. We
                        // just clone it and put it onto the position of the return value, and clears all the other
                        // locals.
                        self.stack[self.frame as usize] = self.stack.top().clone();
                        while self.stack.len() > (self.frame + 1) as usize {
                            self.stack.pop();
                        }
                        self.frame = last_frame.frame;
                        self.closure = last_frame.closure;
                        reader.seek(last_frame.position as usize);
                    } else {
                        break;
                    }
                }

                OperationCode::Print => {
                    // SAFETY: Print can be applied on reference types, and thus we must keep them on stack before
                    // printing to prevent GC to collect them.
                    println!("{}", self.stack.top());
                    self.stack.pop();
                }

                OperationCode::Impossible => unreachable!(),
            }
        }
    }
}
