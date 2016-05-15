use Data;
use Func;
use LibUnsafe;
use SharedlibResult as R;
use std::mem;
use std::path::Path;

/// A shared library which uses bound lifetimes to track its [Symbols](trait.Symbol.html).
#[derive(Debug)]
pub struct Lib {
    inner: LibUnsafe,
}

impl Lib {
    pub fn new<TPath>(path_to_lib: TPath) -> R<Self>
        where TPath: AsRef<Path> {
        let inner = try!(LibUnsafe::new(path_to_lib));
        let result =
            Lib {
                inner: inner,
            };
        Ok(result)
    }

    pub unsafe fn find_data<'a, T, TStr>(&'a self, symbol: TStr) -> R<Data<'a, T>>
        where TStr: AsRef<str> {
        let symbol_ptr = try!(self.inner.find_data::<T, TStr>(symbol));
        let symbol_ref = mem::transmute(symbol_ptr);
        let result = Data::new(symbol_ref);
        Ok(result)
    }

    pub unsafe fn find_func<'a, T, TStr>(&'a self, symbol: TStr) -> R<Func<'a, T>>
        where T: Copy,
              TStr: AsRef<str> {
        let func = try!(self.inner.find_func::<T, TStr>(symbol));
        let result = Func::new(func);
        Ok(result)
    }
}
