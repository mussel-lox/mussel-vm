pub mod bytecode;
pub mod gc;
pub mod value;
pub mod vm;

/// Convenient macro to form a [`bytecode::Bytecode`] quickly and vividly.
#[macro_export]
macro_rules! bytecode {
    (const [$($constant: expr), * $(,)?] $($code: expr); * $(;)?) => {{
        use $crate::bytecode::EmitBytecodeExt;

        let mut bytecode = $crate::bytecode::Bytecode::new();
        let mut writer = $crate::bytecode::BytecodeWriter::new(&mut bytecode);
        $( writer.define($constant)?; )*
        $( writer.emit($code)?; )*
        bytecode
    }};
}
