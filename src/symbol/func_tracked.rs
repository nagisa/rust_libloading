use FuncUnsafe;
use LibUnsafe;

pub struct FuncTracked<T, TLib> {
    func: FuncUnsafe<T>,
    _lib: TLib,
}

impl <T, TLib> FuncTracked<T, TLib>
    where T: Copy,
          TLib: AsRef<LibUnsafe> + Clone {
    pub fn new(func: FuncUnsafe<T>, lib: TLib) -> Self {
        FuncTracked {
            func: func,
            _lib: lib,
        }
    }

    pub unsafe fn get(&self) -> T {
        self.func
    }
}
