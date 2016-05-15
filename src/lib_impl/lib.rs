use Data;
use Func;
use LibUnsafe;
use result::Result as R;
use std::ffi::OsStr;
use std::mem;

/// A shared library which uses bound lifetimes to track its [Symbols](trait.Symbol.html).
#[derive(Debug)]
pub struct Lib {
    inner: LibUnsafe,
}

impl Lib {
    pub fn new<P: AsRef<OsStr>>(filename: P) -> R<Self> {
        let inner = try!(LibUnsafe::new(filename));
        let result =
            Lib {
                inner: inner
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
