use crate::bytecode::{
    Bytecode, BytecodeReader, Constant, FetchBytecodeExt, GlobalStateIndex, OperationCode,
};
use crate::gc::{Allocate, GarbageCollector};
use crate::stack::Stack;
use crate::value::Value;
use anyhow::{bail, Result};

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

                OperationCode::Negate => match self.stack.pop()? {
                    Value::Number(n) => self.stack.push(Value::Number(-n))?,
                    _ => bail!("negate operator `-` can only be applied to numbers"),
                },
                OperationCode::Not => {
                    let boolean: bool = self.stack.pop()?.into();
                    self.stack.push(Value::Boolean(boolean))?;
                }

                OperationCode::Add => {
                    let right = self.stack.pop()?;
                    let left = self.stack.pop()?;
                    match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            self.stack.push(Value::Number(left + right))?;
                        }
                        (Value::String(left), Value::String(right)) => {
                            let concat = self.gc.allocate(format!("{}{}", *left, *right));
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
                    let right = self.stack.pop()?;
                    let left = self.stack.pop()?;
                    self.stack.push(Value::Boolean(left == right))?;
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

                OperationCode::Pop => drop(self.stack.pop()?),

                OperationCode::Print => println!("{}", self.stack.pop()?),

                OperationCode::Impossible => unreachable!(),
            }
        }
        Ok(())
    }
}
