use crate::bytecode::{Bytecode, BytecodeReader, Constant, FetchBytecodeExt, OperationCode};
use crate::value::Value;
use anyhow::{bail, Result};
use astack::{Stack, StackError};

pub const STACK_SIZE: usize = 1024;

/// The Mussel VM.
///
/// A virtual machine stores program states and executes bytecode instructions. As a stack
/// machine, Mussel VM maintains a stack data structure, and stores local variable and does
/// expression evaluation on it.
pub struct VirtualMachine {
    stack: Stack<Value, STACK_SIZE>,
}

impl VirtualMachine {
    /// Create a virtual machine.
    pub fn new() -> Self {
        Self {
            stack: Stack::new(),
        }
    }

    /// Reset the program states, as if the VM is just created and ready to execute bytecode.
    pub fn reset(&mut self) {
        self.stack.clear();
    }

    /// Execute the bytecode.
    ///
    /// Note that the VM is not reset here, since there may be some needs to execute a piece of
    /// bytecode on some existing program states.
    pub fn interpret(&mut self, bytecode: &Bytecode) -> Result<()> {
        let mut reader = BytecodeReader::new(bytecode);
        macro_rules! arithmetic {
            ($operator: tt) => {{
                let left = self.pop()?;
                let right = self.pop()?;
                match (left, right) {
                    (Value::Number(left), Value::Number(right)) => {
                        self.stack.push(Value::Number(left $operator right))?;
                    }
                    _ => bail!("arithmetic {} can only be applied to numbers", stringify!($operator)),
                }
            }};
        }

        loop {
            let opcode = reader.fetch()?;
            match opcode {
                OperationCode::Constant => {
                    let index: u16 = reader.fetch()?;
                    match reader.load(index as usize)? {
                        Constant::Number(n) => self.push(Value::Number(n))?,
                        Constant::String(s) => self.push(Value::String(s))?,
                    }
                }
                OperationCode::Add => arithmetic!(+),
                OperationCode::Subtract => arithmetic!(-),
                OperationCode::Multiply => arithmetic!(*),
                OperationCode::Divide => arithmetic!(/),
                OperationCode::Return => {
                    println!("{:?}", self.pop()?);
                    break;
                }
                OperationCode::Impossible => unreachable!(),
            }
        }
        Ok(())
    }

    /// Pops a [`Value`] from the stack.
    ///
    /// The official `pop`, `pop_panicking` and `pop_unchecked` methods does not match the
    /// form of `push` method. When the VM tries to pop a Value out and fails, an error should be
    /// reported immediately.
    fn pop(&mut self) -> Result<Value> {
        match self.stack.pop() {
            Some(value) => Ok(value),
            None => bail!(StackError::Underflow),
        }
    }

    /// The dual function of `pop`, for aesthetic purposes :)
    fn push(&mut self, value: Value) -> Result<()> {
        Ok(self.stack.push(value)?)
    }
}
