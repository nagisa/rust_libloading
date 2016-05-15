use Symbol;
use std::mem;

/// A pointer to shared data which provides no protection against outliving its library.
pub type DataUnsafe<T> = *const T;

impl <'a, T> Symbol<&'a T> for DataUnsafe<T> {
    unsafe fn get(&self) -> &'a T {
        mem::transmute(*self)
    }
}
