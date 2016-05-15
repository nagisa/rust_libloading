use DataUnsafe;
use FuncUnsafe;
use os::uses::Lib as InnerLib;
use SharedlibResult as R;
use std::mem;
use std::path::Path;
use util;

/// A shared library which does not track its [Symbols](trait.Symbol.html).
#[derive(Debug)]
pub struct LibUnsafe {
    inner: InnerLib,
}

impl LibUnsafe {
    pub fn new<TPath>(path_to_lib: TPath) -> R<Self>
        where TPath: AsRef<Path> {
        let inner = try!(InnerLib::new(path_to_lib));
        let result =
            LibUnsafe {
                inner: inner,
            };
        Ok(result)
    }

    pub unsafe fn find_data<T, TStr>(&self, symbol: TStr) -> R<DataUnsafe<T>>
        where TStr: AsRef<str> {
        match util::null_terminate(&symbol) {
            Some(symbol) => self.inner.find(symbol),
            None => self.inner.find(symbol),
        }
    }

    pub unsafe fn find_func<T, TStr>(&self, symbol: TStr) -> R<FuncUnsafe<T>>
        where T: Copy,
              TStr: AsRef<str> {
        let func =
            match util::null_terminate(&symbol) {
                Some(symbol) => try!(self.inner.find::<u8, _>(symbol)),
                None => try!(self.inner.find::<u8, _>(symbol)),
            };
        let func_ref = &func;
        let result: T = mem::transmute_copy(func_ref);
        Ok(result)
    }
}
