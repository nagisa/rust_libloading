use Symbol;

// A raw pointer to a function from a shared library.
pub type FuncUnsafe<T> = T;

impl <T> Symbol<T> for FuncUnsafe<T>
    where T: Copy {
    unsafe fn get(&self) -> T {
        self.clone()
    }
}
