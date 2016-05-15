use DataTracked;
use FuncTracked;
use LibUnsafe;
use result::Result as R;
use std::ffi::OsStr;

/// A shared library which allows custom tracking methods.
#[derive(Clone, Debug)]
pub struct LibTracked<TLib> {
    inner: TLib,
}

impl <TLib> LibTracked<TLib>
    where TLib: AsRef<LibUnsafe> + Clone + From<LibUnsafe> {
    pub fn new<P: AsRef<OsStr>>(filename: P) -> R<Self> {
        let lib_unsafe = try!(LibUnsafe::new(filename));
        let inner = TLib::from(lib_unsafe);
        let result =
            LibTracked {
                inner: inner
            };
        Ok(result)
    }

    pub unsafe fn find_data<T>(&self, symbol: &[u8]) -> R<DataTracked<T, TLib>> {
        let lib = self.inner.as_ref();
        let symbol_ptr = try!(lib.find_data::<T>(symbol));
        let result = DataTracked::new(symbol_ptr, self.inner.clone());
        Ok(result)
    }

    pub unsafe fn find_func<T>(&self, symbol: &[u8]) -> R<FuncTracked<T, TLib>>
        where T: Copy {
        let lib = self.inner.as_ref();
        let func = try!(lib.find_func::<T>(symbol));
        let result = FuncTracked::new(func, self.inner.clone());
        Ok(result)
    }
}
