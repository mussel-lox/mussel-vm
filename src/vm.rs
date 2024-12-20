use crate::bytecode::{Bytecode, BytecodeReader, Constant, FetchBytecodeExt, OperationCode};
use crate::value::Value;
use anyhow::{bail, Result};
use astack::{Stack, StackError};

pub const STACK_SIZE: usize = 1024;

pub struct VirtualMachine {
    stack: Stack<Value, STACK_SIZE>,
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            stack: Stack::new(),
        }
    }

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

    fn push(&mut self, value: Value) -> Result<()> {
        self.stack.push(value)?;
        Ok(())
    }

    fn pop(&mut self) -> Result<Value> {
        match self.stack.pop() {
            Some(value) => Ok(value),
            None => bail!(StackError::Underflow),
        }
    }
}
