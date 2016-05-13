//! A memory-safer wrapper around system dynamic library primitives.
//!
//! With this library you can load [dynamic libraries](struct.Library.html) and retrieve
//! [symbols](struct.Symbol.html) from the loaded libraries.
//!
//! Less safe platform specific bindings are available in the [`os::platform`](os/index.html)
//! modules.
use std::ffi::OsStr;
use std::fmt;

#[cfg(unix)]
#[macro_use]
extern crate lazy_static;

use self::os as imp;

pub mod os;
mod util;

pub type Result<T> = ::std::io::Result<T>;

/// A dynamically loaded library.
pub struct Library(imp::Library);

impl Library {
    /// Find and load a shared library (module).
    ///
    /// Locations where library is searched for is platform specific and can’t be adjusted
    /// portably.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use shared_lib::Library;
    /// // on Unix
    /// let lib = Library::new("libm.so.6").unwrap();
    /// // on OS X
    /// let lib = Library::new("libm.dylib").unwrap();
    /// // on Windows
    /// let lib = Library::new("msvcrt.dll").unwrap();
    /// ```
    pub fn new<P: AsRef<OsStr>>(filename: P) -> Result<Library> {
        imp::Library::new(filename).map(Library)
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
    /// # use shared_lib::Library;
    /// # let lib = Library::new("libm.so.6").unwrap();
    /// let sin: &extern fn(f64) -> f64 = unsafe {
    ///     lib.get(b"sin\0").unwrap()
    /// };
    /// ```
    ///
    /// A static or TLS variable:
    ///
    /// ```no_run
    /// # use shared_lib::Library;
    /// # let lib = Library::new("libm.so.6").unwrap();
    /// let errno: &*mut u32 = unsafe {
    ///     lib.get(b"errno\0").unwrap()
    /// };
    /// ```
    pub unsafe fn get<T>(&self, symbol: &[u8]) -> Result<&T> {
        self.0.get(symbol)
    }
}

impl fmt::Debug for Library {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(all(unix, not(any(target_os="macos", target_os="ios", target_os="android"))))]
#[test]
fn libm() {
    let lib = Library::new("libm.so.6").unwrap();
    let sin: Symbol<unsafe extern fn(f64) -> f64> = unsafe {
        lib.get(b"sin").unwrap()
    };
    assert!(unsafe { sin(::std::f64::INFINITY) }.is_nan());
    let errno: Symbol<*mut u32> = unsafe {
        lib.get(b"errno").unwrap()
    };
    assert!(unsafe { **errno } != 0);
    unsafe { **errno = 0; }
}
