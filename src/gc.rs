use std::collections::HashMap;

mod reference;
mod types;

pub use reference::*;
pub use types::*;

use crate::value::Value;

pub struct GarbageCollector {
    allocations: Vec<Reference<()>>,
    string_pool: HashMap<String, usize>,
}

impl GarbageCollector {
    pub fn new() -> Self {
        GarbageCollector {
            allocations: Vec::new(),
            string_pool: HashMap::new(),
        }
    }
}

impl Drop for GarbageCollector {
    fn drop(&mut self) {
        for reference in &mut self.allocations {
            #[cfg(feature = "gc-trace")]
            {
                macro_rules! trace_reference {
                    (
                        $r: expr,
                        $($variant: ident <$typ: ident $name: ident> => ($($e:expr), +)); *
                        $(;)?
                    ) => {
                        match $r.kind() {
                            $(
                            AllocationKind::$variant => {
                                let $name: &$typ = $r.downcast().unwrap();
                                eprint!($($e), *);
                            }
                            )*
                        }
                    };
                }

                eprint!("=== GC Trace === Dropped <reference at {:p}>", reference);
                trace_reference!(
                    reference,
                    String   <String s>          => (" \"{}\"", s);
                    Function <FunctionPointer f> => (" <fun position={:#06X} arity={}>", f.position, f.arity);
                    Closure  <Closure c>         => (" <closure position={:#06X} arity={}>", c.position, c.arity);
                    Upvalue  <Value v>           => (" <upvalue {}>", v);
                );
                eprintln!();
            }
            unsafe { reference.finalize() };
        }
    }
}

#[allow(private_bounds)]
pub trait Allocate<T: AllowedAllocationType> {
    fn allocate(&mut self, value: T) -> Reference<T>;
}

/// The allocation of [`String`] is specialized because we'll implement String Interning.
impl Allocate<String> for GarbageCollector {
    fn allocate(&mut self, value: String) -> Reference<String> {
        if let Some(index) = self.string_pool.get(&value) {
            return unsafe { self.allocations[*index].cast() };
        }
        let allocation = unsafe { Reference::spawn(AllocationKind::String, value.clone()) };
        self.string_pool.insert(value, self.allocations.len());
        self.allocations.push(unsafe { allocation.cast() });
        allocation
    }
}

#[allow(unused_macros)]
macro_rules! allocate_impl {
    ($($variant: ident => $t: ty); * $(;)?) => {
        $(
        impl Allocate<$t> for GarbageCollector {
            fn allocate(&mut self, value: $t) -> Reference<$t> {
                let allocation = unsafe { Reference::spawn(AllocationKind::$variant, value) };
                self.allocations.push(unsafe { allocation.cast() });
                allocation
            }
        }
        )*
    };
}

allocate_impl! {
    Function => FunctionPointer;
    Closure => Closure;
    Upvalue => Value;
}
