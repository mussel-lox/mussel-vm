use std::{
    mem,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    slice::{Iter, IterMut},
};

use anyhow::{bail, Result};

/// The Stack data structure.
///
/// This struct is adopted because the [`Vec`] is allocated on heap. More specifically, it's
/// actually a `(ptr, len, cap)` triplet. Instead, this [`Stack`] allocates on stack (against
/// heap), and exposes safe API such as `push`, `pop` and [`Deref`] impls.
pub struct Stack<T, const N: usize> {
    /// The stack elements
    ///
    /// Elements are stored in an array in order to keep them on stack (against heap). Therefore,
    /// the const generic value [`N`] is needed.
    ///
    /// [`MaybeUninit`] is introduced since Rust does not allow creating an array without
    /// providing any default value. The memory layout of [`MaybeUninit`] is guaranteed the same
    /// with [`T`], so there's no extra memory cost.
    elements: [MaybeUninit<T>; N],
    top: usize,
}

impl<T, const N: usize> Stack<T, N> {
    /// Create an empty stack.
    pub const fn new() -> Self {
        Self {
            elements: [const { MaybeUninit::uninit() }; N],
            top: 0,
        }
    }

    /// Returns the length of stack. The capacity is always [`N`] by the way.
    pub fn len(&self) -> usize {
        self.top
    }

    /// Returns whether the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.top == 0
    }

    /// Get a reference to the top `n`-th element of the stack if any.
    ///
    /// When we talk about the "top" concept in the [`Stack`], we're talking about the last
    /// pushed elements. The last pushed elements are placed on top of those first pushed ones,
    /// since the first ones cannot to be taken out without removing the last ones. That's vivid.
    ///
    /// Peek operation is very useful, especially in a stack machine. We can check the operands
    /// in-place, without memory cost of `pop`ping and `push`ing them back and forth.
    ///
    /// `n` must be in the range of the range `[0, len)`. When `n` == `0`, a reference of the top
    /// element is returned, if any.
    pub fn peek(&self, n: usize) -> Result<&T> {
        if self.len() <= n {
            bail!("stack underflow, cannot peek the top {}-th element", n);
        }
        Ok(unsafe { self.elements[self.top - n - 1].assume_init_ref() })
    }

    /// Gets the top element. It's a special form of `peek`.
    pub fn top(&self) -> Result<&T> {
        self.peek(0)
    }

    /// Pushes a value into the stack.
    ///
    /// If the stack overflows, an [`anyhow::Error`] is reported.
    pub fn push(&mut self, value: T) -> Result<()> {
        if self.len() >= N {
            bail!("stack overflow");
        }
        self.elements[self.top].write(value);
        self.top += 1;
        Ok(())
    }

    /// Pops a value out of the stack.
    ///
    /// If the stack underflows, an [`anyhow::Error`] is returned.
    pub fn pop(&mut self) -> Result<T> {
        if self.is_empty() {
            bail!("stack underflow");
        }
        self.top -= 1;
        let value = unsafe { self.elements[self.top].assume_init_read() };
        Ok(value)
    }

    /// Dropping every element, and sets the stack top to the first slot.
    ///
    /// Since we're using [`MaybeUninit`], we just call `assume_init_drop` to drop the elements
    /// in-place. This should save some memory cost :).
    pub fn clear(&mut self) {
        for element in (&mut self.elements[..self.top]).into_iter().rev() {
            unsafe { element.assume_init_drop() };
        }
        self.top = 0;
    }
}

impl<T, const N: usize> Drop for Stack<T, N> {
    fn drop(&mut self) {
        self.clear()
    }
}

impl<T, const N: usize> Deref for Stack<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { mem::transmute(&self.elements[..self.top]) }
    }
}

impl<T, const N: usize> DerefMut for Stack<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { mem::transmute(&mut self.elements[..self.top]) }
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a Stack<T, N> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.deref().into_iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut Stack<T, N> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.deref_mut().into_iter()
    }
}
