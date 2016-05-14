use os::uses::Lib as InnerLib;
use result::Result as R;
use std::ffi::OsStr;
use std::mem;
use Data;
use Func;

#[derive(Debug)]
pub struct Lib {
    inner: InnerLib,
}

impl Lib {
    /// Find and load a shared library (module).
    ///
    /// Locations where library is searched for is platform specific and can’t be adjusted
    /// portably.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use sharedlib::Library;
    /// // on Unix
    /// let lib = Library::new("libm.so.6").unwrap();
    /// // on OS X
    /// let lib = Library::new("libm.dylib").unwrap();
    /// // on Windows
    /// let lib = Library::new("msvcrt.dll").unwrap();
    /// ```
    pub fn new<P: AsRef<OsStr>>(filename: P) -> R<Lib> {
        InnerLib::new(filename)
            .map(|inner| Lib { inner: inner })
    }

    /// Get a symbol by name.
    ///
    /// Mangling or symbol rustification is not done: trying to `get` something like `x::y`
    /// will not work.
    ///
    /// You may append a null byte at the end of the byte string to avoid string allocation in some
    /// cases. E.g. for symbol `sin` you may write `b"sin\0"` instead of `b"sin"`.
    ///
    /// # Unsafety
    ///
    /// Symbol of arbitrary requested type is returned. Using a symbol with wrong type is not
    /// memory safe.
    ///
    /// # Examples
    ///
    /// Simple function:
    ///
    /// ```no_run
    /// # use sharedlib::Library;
    /// # let lib = Library::new("libm.so.6").unwrap();
    /// let sin: &extern fn(f64) -> f64 = unsafe {
    ///     lib.get(b"sin\0").unwrap()
    /// };
    /// ```
    ///
    /// A static or TLS variable:
    ///
    /// ```no_run
    /// # use sharedlib::Library;
    /// # let lib = Library::new("libm.so.6").unwrap();
    /// let errno: &*mut u32 = unsafe {
    ///     lib.get(b"errno\0").unwrap()
    /// };
    /// ```
    pub unsafe fn get_data_unsafe<T>(&self, symbol: &[u8]) -> R<*const T> {
        self.inner.get(symbol)
    }

    pub unsafe fn get_func_unsafe<T>(&self, symbol: &[u8]) -> R<T>
        where T: Copy {
        let func = try!(self.inner.get::<u8>(symbol));
        let func_ref = &func;
        let result: T = mem::transmute_copy(func_ref);
        Ok(result)
    }

    pub unsafe fn get_data<'a, T>(&'a self, symbol: &[u8]) -> R<Data<'a, T>> {
        let symbol_ptr = try!(self.get_data_unsafe::<T>(symbol));
        let symbol_ref = mem::transmute(symbol_ptr);
        let result = Data::new(symbol_ref);
        Ok(result)
    }

    pub unsafe fn get_func<'a, T>(&'a self, symbol: &[u8]) -> R<Func<'a, T>>
        where T: Copy {
        let func = try!(self.get_func_unsafe::<T>(symbol));
        let result = Func::new(func);
        Ok(result)
    }
}
