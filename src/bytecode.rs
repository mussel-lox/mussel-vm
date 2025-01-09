use std::hash::{Hash, Hasher};

use byteorder::LittleEndian;

mod reader;
mod writer;

pub use reader::*;
pub use writer::*;

/// The endianness of bytecode. Used in [`BytecodeReader`] and [`BytecodeWriter`].
pub type Endianness = LittleEndian;
/// The type of constant index in a [`Bytecode`]. Defined using typedef to deal with possible changes in the future.
pub type ConstantIndex = u16;
/// The type of global states' (i.e. variables) index.
pub type GlobalIndex = u8;
/// The type of locals' index (i.e. the stack index).
pub type LocalOffset = u8;
/// The type of the jump offset. It can be negative to indicating jumping backward.
pub type JumpOffset = i16;
/// The type representing an absolute index of a function entry.
pub type CallPosition = u16;

/// The operation codes.
///
/// Operation codes forms the virtual ISA, which is recognized by the virtual machine (VM). It's a dense, linear
/// sequence of instruction and is good for performance. Tree structures at the source code level (e.g. control
/// flows) are implemented by several kinds of jump instructions.
#[repr(u8)]
pub enum OperationCode {
	/// Load a constant into the VM stack, with its index stored as [`ConstantIndex`] following the operation code.
	Constant,
	Nil,
	True,
	False,
	/// Create a function pointer, according to the following [`CallPosition`] and [`LocalOffset`] (arity).
	Fun,

	Negate,
	Not,

	Add,
	Subtract,
	Multiply,
	Divide,

	Equal,
	Greater,
	Less,

	/// Gets the specified global variable, and push it into the stack. Same as the `SetGlobal` operation code, this
	/// code is followed by a [`GlobalIndex`].
	GetGlobal,
	/// Pops the top element of the stack, and sets it as a global state (i.e. variable) with its index in
	/// [`GlobalIndex`] type.
	SetGlobal,

	/// Gets the specified slot of stack and pushes the value at the top of it. This code is followed by a
	/// [`LocalOffset`], which is an offset starts from the current call frame.
	GetLocal,
	/// Pops the specified slot with a [`LocalOffset`] offset starts from the current call frame and pushes the value
	/// at the top of the stack.
	SetLocal,
	/// Simply pops and drops the top element of the stack.
	Pop,

	/// Create a closure object based on a [`CallPosition`] and the arity in [`LocalOffset`] type.
	Closure,
	/// Box a value on stack with position [`LocalOffset`] as upvalue if never boxed, and bind it to the closure
	/// object at the stack top.
	Capture,
	/// Get an upvalue at a certain position in [`LocalOffset`] type of the current closure.
	GetUpvalue,
	/// Sets the value at the stack top to the upvalue at position in [`LocalOffset`] type.
	SetUpvalue,

	/// Jumps according to the following [`JumpOffset`] if the top element of the current stack can be evaluated as
	/// false. The offset can be positive or negative, in order to jump forward or backward.
	JumpIfFalse,
	/// Instantly jumps according to the following [`JumpOffset`]. There's no conditions to meet.
	Jump,
	/// Start a new call frame, and instantly jumps to the absolute position.
	///
	/// This is a two-operand code. It receives a [`CallPosition`] representing the absolute position of the function
	/// entry, and a [`LocalOffset`] indicating the start of the call frame from the stack top.
	Call,
	/// Invokes a function pointer, or a closure.
	///
	/// This is similar to [`OperationCode::Call`], but no operands are needed. This code pops the top element of the
	/// stack and calls it.
	Invoke,
	/// Return to the outer function call.
	///
	/// More specifically, if there is an outer function, the value at the stack top will be preserved as the return
	/// value, and all the other local variables will be dropped. The VM will jump back to the last function's
	/// position and continues to execute it.
	///
	/// If there's no such an outer function (i.e. this is the "main" function), the VM just exits.
	Return,

	Print,

	/// Guard variant to detect invalid operation codes.
	Impossible,
}

/// The constants stored in a [`Bytecode`].
///
/// The Mussel VM recognizes some value types (numbers, strings, booleans, object type and nil). However, not all of
/// them can be stored in [`Bytecode`] as constants. For object types, dynamic memory allocation is needed, which is
/// a runtime feature; for boolean and nil literals, there are corresponding operation codes designed to push a value
/// into the VM stack.
///
/// For now, only numbers (internally `f64`) and strings (internally [`String`]) are considered as constants.
#[derive(Debug, Clone)]
pub enum Constant {
	Number(f64),
	String(String),
}

impl Hash for Constant {
	fn hash<H: Hasher>(&self, state: &mut H) {
		match self {
			Constant::Number(n) => n.to_bits().hash(state),
			Constant::String(s) => s.hash(state),
		}
	}
}

impl PartialEq for Constant {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Constant::Number(n1), Constant::Number(n2)) => (n1 - n2).abs() < f64::EPSILON,
			(Constant::String(s1), Constant::String(s2)) => s1 == s2,
			_ => false,
		}
	}
}

impl Eq for Constant {}

/// The bytecode.
///
/// Bytecode is the binary representation of a program. As Niklaus Wirth describes, the bytecode is also the
/// combination of data structure and algorithms. More specifically, a [`Bytecode`] of Mussel VM consists of a
/// [`OperationCode`] sequence and some [`Constant`]s.
pub struct Bytecode {
	pub code: Vec<u8>,
	pub constants: Vec<Constant>,
}
