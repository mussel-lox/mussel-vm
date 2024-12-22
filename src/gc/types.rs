use crate::bytecode::{CallPosition, LocalOffset};

#[derive(Debug, Hash)]
pub struct FunctionPointer {
    pub position: CallPosition,
    pub arity: LocalOffset,
}
