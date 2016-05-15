use Symbol;

/// A pointer to a shared function which provides no protection against outliving its library.
pub type FuncUnsafe<T> = T;

impl <T> Symbol<T> for FuncUnsafe<T>
    where T: Copy {
    unsafe fn get(&self) -> T {
        self.clone()
    }
}
