use anyhow::{bail, Result};

use crate::{
    bytecode::{
        Bytecode, BytecodeReader, Constant, FetchBytecodeExt, GlobalStateIndex, OperationCode,
    },
    gc::{Allocate, GarbageCollector},
    stack::Stack,
    value::Value,
};

pub const STACK_SIZE: usize = 1024;

/// The Mussel VM.
///
/// A virtual machine stores program states and executes bytecode instructions. As a stack
/// machine, Mussel VM maintains a stack data structure, and stores local variable and does
/// expression evaluation on it.
pub struct VirtualMachine {
    globals: Vec<Value>,
    stack: Stack<Value, STACK_SIZE>,
    gc: GarbageCollector,
}

impl VirtualMachine {
    /// Create a virtual machine.
    pub fn new() -> Self {
        Self {
            globals: Vec::new(),
            stack: Stack::new(),
            gc: GarbageCollector::new(),
        }
    }

    /// Reset the program states, as if the VM is just created and ready to execute bytecode.
    ///
    /// Note that GC is not reset here, it's up to itself to collect garbage.
    pub fn reset(&mut self) {
        self.globals.clear();
        self.stack.clear();
    }

    /// Execute the bytecode.
    ///
    /// Note that the VM is not reset here, since there may be some needs to execute a piece of
    /// bytecode on some existing program states.
    pub fn interpret(&mut self, bytecode: &Bytecode) -> Result<()> {
        let mut reader = BytecodeReader::new(bytecode);
        macro_rules! arithmetic {
            ($operator: tt as $variant: ident) => {{
                /// SAFETY: The arithmetic operations can only be applied to numbers. When the
                /// operands are reference types, an error will be instantly reported, leaving
                /// the GC behavior unimportant. Thus, we don't need to keep the values on stack
                /// before evaluation.
                let right = self.stack.pop()?;
                let left = self.stack.pop()?;
                match (left, right) {
                    (Value::Number(left), Value::Number(right)) => {
                        self.stack.push(Value::$variant(left $operator right))?;
                    }
                    _ => bail!(
                        "arithmetic operator `{}` can only be applied to numbers",
                        stringify!($operator),
                    ),
                }
            }};
        }

        loop {
            let opcode = reader.fetch()?;
            match opcode {
                OperationCode::Constant => {
                    let index: u16 = reader.fetch()?;
                    match reader.load(index as usize)? {
                        Constant::Number(n) => self.stack.push(Value::Number(n))?,
                        Constant::String(s) => {
                            let allocation = self.gc.allocate(s);
                            self.stack.push(Value::String(allocation))?;
                        }
                    }
                }
                OperationCode::Nil => self.stack.push(Value::Nil)?,
                OperationCode::True => self.stack.push(Value::Boolean(true))?,
                OperationCode::False => self.stack.push(Value::Boolean(false))?,

                /// SAFETY: Negate operation can only be applied on numbers. When the operand is
                /// of reference type, an error will be reported instantly, leaving the GC
                /// behavior unimportant. Thus, we don't have to keep the operand on stack before
                /// evaluation.
                OperationCode::Negate => match self.stack.pop()? {
                    Value::Number(n) => self.stack.push(Value::Number(-n))?,
                    _ => bail!("negate operator `-` can only be applied to numbers"),
                },
                OperationCode::Not => {
                    /// SAFETY: Not operation indeed can be applied to all value types, including
                    /// the reference types. However, whether the converted value true or false
                    /// is irrelevant to the validity of the reference, and no dereferencing is
                    /// performed. Thus, the value doesn't need to be on stack before evaluation.
                    let boolean: bool = self.stack.pop()?.into();
                    self.stack.push(Value::Boolean(boolean))?;
                }

                OperationCode::Add => {
                    /// SAFETY: Add operation can be applied to numbers or strings, and the
                    /// latter is a reference type. We'll have to keep the reference values on
                    /// stack before evaluation since we cannot know when the GC will execute.
                    let right = self.stack.peek(0)?;
                    let left = self.stack.peek(1)?;
                    match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            self.stack.pop()?;
                            self.stack.pop()?;
                            self.stack.push(Value::Number(left + right))?;
                        }
                        (Value::String(left), Value::String(right)) => {
                            let concat = self.gc.allocate(format!("{}{}", **left, **right));
                            self.stack.pop()?;
                            self.stack.pop()?;
                            self.stack.push(Value::String(concat))?;
                        }
                        _ => bail!("add operator `+` can only be applied to numbers or strings"),
                    }
                }
                OperationCode::Subtract => arithmetic!(- as Number),
                OperationCode::Multiply => arithmetic!(* as Number),
                OperationCode::Divide => arithmetic!(/ as Number),
                OperationCode::Return => break,

                OperationCode::Equal => {
                    /// SAFETY: Equal operation can be applied to each kind of values, and
                    /// there's reference types. We'll have to keep the reference values on stack
                    /// before evaluation since we cannot know when the GC will execute.
                    let right = self.stack.peek(0)?;
                    let left = self.stack.peek(1)?;
                    let equal = Value::Boolean(left == right);
                    self.stack.pop()?;
                    self.stack.pop()?;
                    self.stack.push(equal)?;
                }
                OperationCode::Greater => arithmetic!(> as Boolean),
                OperationCode::Less => arithmetic!(< as Boolean),

                OperationCode::SetGlobal => {
                    let index: GlobalStateIndex = reader.fetch()?;
                    if index as usize >= self.globals.len() {
                        self.globals.resize(index as usize + 1, Value::Nil);
                    }
                    self.globals[index as usize] = self.stack.top()?.clone();
                }
                OperationCode::GetGlobal => {
                    let index: GlobalStateIndex = reader.fetch()?;
                    if index as usize >= self.globals.len() {
                        bail!("global index {} out of bounds", index);
                    }
                    self.stack.push(self.globals[index as usize].clone())?
                }

                /// No SAFETY here because the Pop operation means to pop a value out of
                /// stack directly.
                OperationCode::Pop => drop(self.stack.pop()?),

                OperationCode::Print => {
                    /// SAFETY: Print can be applied on reference types, and thus we must keep
                    /// them on stack before printing to prevent GC to collect them.
                    println!("{}", self.stack.top()?);
                    self.stack.pop()?;
                }

                OperationCode::Impossible => unreachable!(),
            }
        }
        Ok(())
    }
}
