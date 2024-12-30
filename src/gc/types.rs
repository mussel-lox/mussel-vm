use crate::{
    bytecode::{CallPosition, LocalOffset},
    gc::Reference,
    value::Value,
};

#[derive(Debug)]
pub struct FunctionPointer {
    pub position: CallPosition,
    pub arity: LocalOffset,
}

pub type Upvalue = Reference<Value>;

#[derive(Debug)]
pub struct Closure {
    pub position: CallPosition,
    pub arity: LocalOffset,
    pub upvalues: Vec<Upvalue>,
}
