use Symbol;
use std::mem;

pub type DataUnsafe<T> = *const T;

impl <'a, T> Symbol<&'a T> for DataUnsafe<T> {
    unsafe fn get(&self) -> &'a T {
        mem::transmute(*self)
    }
}
