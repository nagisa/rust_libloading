use Symbol;
use std::mem;

// A raw pointer to some data from a shared library.
pub type DataUnsafe<T> = *const T;

impl <'a, T> Symbol<&'a T> for DataUnsafe<T> {
    unsafe fn get(&self) -> &'a T {
        mem::transmute(*self)
    }
}
