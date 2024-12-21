use anyhow::{bail, Result};
use std::mem;
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::slice::{Iter, IterMut};

pub struct Stack<T, const N: usize> {
    elements: [MaybeUninit<T>; N],
    top: usize,
}

impl<T, const N: usize> Stack<T, N> {
    pub const fn new() -> Self {
        Self {
            elements: [const { MaybeUninit::uninit() }; N],
            top: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.top
    }

    pub fn is_empty(&self) -> bool {
        self.top == 0
    }

    pub fn peek(&self, n: usize) -> Result<&T> {
        if self.len() <= n {
            bail!("stack underflow, cannot peek the top {}-th element", n);
        }
        Ok(unsafe { self.elements[self.top - n - 1].assume_init_ref() })
    }

    pub fn top(&self) -> Result<&T> {
        self.peek(0)
    }

    pub fn push(&mut self, value: T) -> Result<()> {
        if self.len() >= N {
            bail!("stack overflow");
        }
        self.elements[self.top].write(value);
        self.top += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Result<T> {
        if self.is_empty() {
            bail!("stack underflow");
        }
        self.top -= 1;
        let value = unsafe { self.elements[self.top].assume_init_read() };
        Ok(value)
    }

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
