use std::marker::PhantomData;

pub struct Func<'a, T>
    where T: Copy + 'a {
    data: T,
    lifetime: PhantomData<&'a ()>,
}

impl <'a, T> Func<'a, T>
    where T: Copy + 'a {
    pub unsafe fn new(data: T) -> Self {
        Func {
            data: data,
            lifetime: PhantomData,
        }
    }

    pub unsafe fn get(&self) -> T {
        self.data
    }
}
