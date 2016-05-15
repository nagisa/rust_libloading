use DataTracked;
use FuncTracked;
use LibUnsafe;
use SharedlibResult as R;
use std::path::Path;

/// A shared library which which allows a user-provided ref-counting implementation to track its [Symbols](trait.Symbol.html).
#[derive(Clone, Debug)]
pub struct LibTracked<TLib> {
    inner: TLib,
}

impl <TLib> LibTracked<TLib>
    where TLib: AsRef<LibUnsafe> + Clone + From<LibUnsafe> {
    pub fn new<TPath>(path_to_lib: TPath) -> R<Self>
        where TPath: AsRef<Path> {
        let lib_unsafe = try!(LibUnsafe::new(path_to_lib));
        let inner = TLib::from(lib_unsafe);
        let result =
            LibTracked {
                inner: inner
            };
        Ok(result)
    }

    pub unsafe fn find_data<T, TStr>(&self, symbol: TStr) -> R<DataTracked<T, TLib>>
        where TStr: AsRef<str> {
        let lib = self.inner.as_ref();
        let symbol_ptr = try!(lib.find_data::<T, TStr>(symbol));
        let result = DataTracked::new(symbol_ptr, self.inner.clone());
        Ok(result)
    }

    pub unsafe fn find_func<T, TStr>(&self, symbol: TStr) -> R<FuncTracked<T, TLib>>
        where T: Copy,
              TStr: AsRef<str> {
        let lib = self.inner.as_ref();
        let func = try!(lib.find_func::<T, TStr>(symbol));
        let result = FuncTracked::new(func, self.inner.clone());
        Ok(result)
    }
}
