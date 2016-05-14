use FuncUnsafe;
use std::marker::PhantomData;

pub struct Func<'a, T>
    where T: Copy + 'a {
    func: FuncUnsafe<T>,
    lifetime: PhantomData<&'a ()>,
}

impl <'a, T> Func<'a, T>
    where T: Copy + 'a {
    pub fn new(func: FuncUnsafe<T>) -> Self {
        Func {
            func: func,
            lifetime: PhantomData,
        }
    }

    pub unsafe fn get(&self) -> T {
        self.func
    }
}
