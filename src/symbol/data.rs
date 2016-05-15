use Symbol;

// A pointer to data in a shared library which uses a lifetime to avoid outliving the library.
pub struct Data<'a, T>
    where T: 'a {
    data: &'a T,
}

impl <'a, T> Data<'a, T> {
    pub fn new(data: &'a T) -> Self {
        Data {
            data: data,
        }
    }
}

impl <'a, T> Symbol<&'a T> for Data<'a, T> {
    unsafe fn get(&self) -> &'a T {
        self.data
    }
}
