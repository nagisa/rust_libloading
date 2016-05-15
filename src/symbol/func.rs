use FuncUnsafe;
use std::marker::PhantomData;
use Symbol;

/// A pointer to a shared function which uses a bound lifetime to avoid outliving its library.
pub struct Func<'a, T> {
    func: FuncUnsafe<T>,
    lifetime: PhantomData<&'a ()>,
}

impl <'a, T> Func<'a, T> {
    pub fn new(func: FuncUnsafe<T>) -> Self {
        Func {
            func: func,
            lifetime: PhantomData,
        }
    }
}

impl <'a, T> Symbol<T> for Func<'a, T>
    where T: Copy {
    unsafe fn get(&self) -> T {
        self.func
    }
}
