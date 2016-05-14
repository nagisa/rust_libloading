use Symbol;

pub type FuncUnsafe<T> = T;

impl <T> Symbol<T> for FuncUnsafe<T>
    where T: Copy {
    unsafe fn get(&self) -> T {
        self.clone()
    }
}
