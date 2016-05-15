use DataUnsafe;
use FuncUnsafe;
use os::uses::Lib as InnerLib;
use result::Result as R;
use std::ffi::OsStr;
use std::mem;
use util;

/// A shared library which does not track its [Symbols](trait.Symbol.html).
#[derive(Debug)]
pub struct LibUnsafe {
    inner: InnerLib,
}

impl LibUnsafe {
    /// Find and load a shared library.
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
    pub fn new<P: AsRef<OsStr>>(filename: P) -> R<Self> {
        InnerLib::new(filename)
            .map(|inner| LibUnsafe { inner: inner })
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
