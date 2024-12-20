use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::ptr::NonNull;

/// Helper trait to limit a generic type parameter to a range of GC allowed allocation types.
pub(super) trait AllowedAllocationType {}

/// The raw allocation of a specific type.
///
/// When GC allocates something on the heap, some extra information (i.e. "object header") is
/// included. This struct is the combination of those extra fields with the value type [`T`].
///
/// This struct will be used in a `NonNull<RawAllocation<T>>` form, which is a thin pointer and
/// reaching the metadata or the value itself only requires dereferencing the pointer once.
///
/// `#[repr(C)]` is used to prevent Rustc from changing the layout of allocation, which will
/// cause undefined behavior.
#[repr(C)]
#[derive(Debug)]
pub(super) struct RawAllocation<T> {
    typ: AllocationType,
    value: T,
}

/// A pointer to a chunk of GC allocation.
///
/// This is a thin pointer, which may improve performance when doing value copying in Mussel VM.
/// The pointer points at a [`RawAllocation`] object, which contains both metadata and the value
/// itself.
///
/// When the type parameter [`T`] is `()`, [`Downcast`] trait is implemented. This is necessary
/// for GC to recognize the actual types of allocations, and perform the correct clean-up action.
#[derive(Debug)]
pub struct Reference<T>(NonNull<RawAllocation<T>>);

impl<T> Reference<T> {
    /// Allocate a chunk of memory, returning its reference.
    ///
    /// A helper trait [`AllowedAllocationType`] is applied to limit the value type [`T`] in a
    /// valid range. However. this function is still marked with `unsafe` because the other parts
    /// of code might get the [`AllocationType`] wrong. Check `unsafe` code carefully!
    #[allow(private_bounds)]
    #[allow(private_interfaces)]
    pub unsafe fn spawn(typ: AllocationType, value: T) -> Self
    where
        T: AllowedAllocationType,
    {
        Self(NonNull::new_unchecked(Box::into_raw(Box::new(RawAllocation { typ, value }))).cast())
    }

    /// Cast a reference from type [`T`] to type [`U`].
    ///
    /// This is extremely unsafe since we cannot ensure if the casting is correct.
    pub unsafe fn cast<U>(self) -> Reference<U> {
        Reference(self.0.cast())
    }

    pub fn typ(&self) -> AllocationType {
        unsafe { self.0.as_ref().typ }
    }
}

impl<T> Deref for Reference<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &self.0.as_ref().value }
    }
}

impl<T> DerefMut for Reference<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut self.0.as_mut().value }
    }
}

impl<T> Clone for Reference<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T> Copy for Reference<T> {}

impl<T, U> PartialEq<Reference<U>> for Reference<T> {
    fn eq(&self, other: &Reference<U>) -> bool {
        ptr::addr_eq(self.0.as_ptr(), other.0.as_ptr())
    }
}

impl<T> Eq for Reference<T> {}

impl<T: Display + AllowedAllocationType> Display for Reference<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}

/// The helper trait to perform downcasting on a [`Reference`].
///
/// This trait is safe: when the underlying type is [`T`], it returns some reference; otherwise,
/// [`None`] is returned. The principle is simple: we just check the metadata (i.e.
/// [`AllocationType`]) in the [`RawAllocation`], and returns the reference if matches.
pub trait Downcast<T> {
    /// Returns an immutable reference of [`T`] if the type matches, [`None`] is returned otherwise.
    fn downcast(&self) -> Option<&T>;

    /// Returns a mutable reference of [`T`] if the type matches, [`None`] is returned otherwise.
    fn downcast_mut(&mut self) -> Option<&mut T>;
}

/// This is the magic part: since there exist a lot of duplicate code segment, we just take
/// advantage of the macro to generate them for us automatically.
///
/// Moreover, because of the exhaustibility of allowed allocations types, we can ensure the type
/// safety by doing so -- we don't provide corresponding functions for other types.
macro_rules! register_allowed_types {
    ($($variant: ident => $t: ty); * $(;)?) => {
        /// The metadata to recognize the actual type of an allocation.
        #[repr(C)]
        #[derive(Debug, Clone, Copy)]
        pub(super) enum AllocationType {
            $($variant), *
        }

        $(
        impl AllowedAllocationType for $t {}

        impl Downcast<$t> for Reference<()> {
            fn downcast(&self) -> Option<&$t> {
                let allocation = unsafe { self.0.as_ref() };
                #[allow(unreachable_patterns)]
                match allocation.typ {
                    AllocationType::$variant => Some(unsafe { &*(self as *const _ as *const $t) }),
                    _ => None
                }
            }

            fn downcast_mut(&mut self) -> Option<&mut $t> {
                let allocation = unsafe { self.0.as_ref() };
                #[allow(unreachable_patterns)]
                match allocation.typ {
                    AllocationType::$variant => Some(unsafe { &mut *(self as *mut _ as *mut $t) }),
                    _ => None
                }
            }
        }
        )*

        impl Reference<()> {
            /// Finalize a reference.
            ///
            /// We cannot rely on RAII pattern or borrow checker to clean up the resource, the GC
            /// algorithm is against that. Instead, the GC will analyze the reachability of every
            /// allocation, and finalize them manually at the right time.
            ///
            /// This function is unsafe because, if we've written the wrong algorithm, multiple dropping
            /// of an allocation is possible. And `unsafe` keyword is of this use: the marked code should
            /// be checked very carefully.
            ///
            /// This function is only implemented on `Reference<()>`, because it's the type
            /// adopted by GC. Any other parts of the program should not finalize any single
            /// reference.
            pub unsafe fn finalize(&mut self) {
                let allocation = unsafe { self.0.as_ref() };
                match allocation.typ {
                    $(
                    AllocationType::$variant => self.0.cast::<RawAllocation<$t>>().drop_in_place(),
                    )*
                }
            }
        }

        impl Display for Reference<()> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                match self.typ() {
                    $(
                    AllocationType::$variant => unsafe { self.cast::<$t>().fmt(f) },
                    )*
                }
            }
        }
    };
}

register_allowed_types! {
    String => String;
}