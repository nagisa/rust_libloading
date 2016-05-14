pub struct Data<'a, T>
    where T: 'a {
    data: &'a T,
}

impl <'a, T> Data<'a, T>
    where T: 'a {
    pub fn new(data: &'a T) -> Self {
        Data {
            data: data,
        }
    }

    pub unsafe fn get(&self) -> &T {
        self.data
    }
}
