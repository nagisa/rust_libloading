use DataUnsafe;
use LibUnsafe;
use std::mem;

#[derive(Clone)]
pub struct DataTracked<T, TLib> {
    data: DataUnsafe<T>,
    _lib: TLib,
}

impl <T, TLib> DataTracked<T, TLib>
    where TLib: AsRef<LibUnsafe> + Clone {
    pub fn new(data: DataUnsafe<T>, lib: TLib) -> Self {
        DataTracked {
            data: data,
            _lib: lib,
        }
    }

    pub unsafe fn get(&self) -> &T {
        mem::transmute(self.data)
    }
}
