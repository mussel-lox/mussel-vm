use crate::bytecode::{Bytecode, Constant, OperationCode};
use crate::value::Value;
use anyhow::{bail, Result};
use astack::{Stack, StackError};
use std::ptr::NonNull;

pub const STACK_SIZE: usize = 1024;

pub struct VirtualMachine {
    bytecode: Option<NonNull<Bytecode>>,
    offset: usize,
    stack: Stack<Value, STACK_SIZE>,
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            bytecode: None,
            offset: 0,
            stack: Stack::new(),
        }
    }

    pub fn interpret(&mut self, bytecode: &Bytecode) -> Result<()> {
        self.bytecode = Some(NonNull::from(bytecode));
        self.offset = 0;
        self.run()
    }

    fn run(&mut self) -> Result<()> {
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

        // SAFETY: this function should only be call by self.interpret, during whose lifetime the
        // reference of a Bytecode is guaranteed to be alive.
        let bytecode = unsafe { self.bytecode.unwrap().as_ref() };
        loop {
            if self.offset >= bytecode.code.len() {
                bail!("bytecode out of bounds and does not return");
            }
            let opcode = OperationCode::try_from(bytecode.code[self.offset])?;
            match opcode {
                OperationCode::Constant => {
                    let (index, size) = bytecode.extract_u16(self.offset)?;
                    match &bytecode.constants[index as usize] {
                        Constant::Number(n) => self.push(Value::Number(*n))?,
                        Constant::String(s) => self.push(Value::String(s.clone()))?,
                    }
                    self.offset += size;
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
            self.offset += 1;
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
