use FuncUnsafe;
use Symbol;

/// A pointer to a shared function which allows a user-provided ref-counting implementation to avoid outliving its library.
pub struct FuncTracked<T, TLib> {
    func: FuncUnsafe<T>,
    _lib: TLib,
}

impl <T, TLib> FuncTracked<T, TLib> {
    pub fn new(func: FuncUnsafe<T>, lib: TLib) -> Self {
        FuncTracked {
            func: func,
            _lib: lib,
        }
    }
}

impl <T, TLib> Symbol<T> for FuncTracked<T, TLib>
    where T: Copy {
    unsafe fn get(&self) -> T {
        self.func
    }
}
