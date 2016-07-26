//! A memory-safer wrapper around system dynamic library primitives.
//!
//! With this library you can load [dynamic libraries](struct.Library.html) and retrieve
//! [symbols](struct.Symbol.html) from the loaded libraries.
//!
//! Less safe platform specific bindings are available in the [`os::platform`](os/index.html)
//! modules.
use std::ffi::OsStr;
use std::fmt;
use std::marker;

#[cfg(unix)]
#[macro_use]
extern crate lazy_static;

#[cfg(unix)]
use self::os::unix as imp;
#[cfg(windows)]
use self::os::windows as imp;

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
    /// # use ::libloading::Library;
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
    /// # use ::libloading::{ Library, Symbol };
    /// # let lib = Library::new("libm.so.6").unwrap();
    /// let sin: Symbol<unsafe extern fn(f64) -> f64> = unsafe {
    ///     lib.get(b"sin\0").unwrap()
    /// };
    /// ```
    ///
    /// A static or TLS variable:
    ///
    /// ```no_run
    /// # use ::libloading::{ Library, Symbol };
    /// # let lib = Library::new("libm.so.6").unwrap();
    /// let errno: Symbol<*mut u32> = unsafe {
    ///     lib.get(b"errno\0").unwrap()
    /// };
    /// ```
    pub unsafe fn get<'lib, T>(&'lib self, symbol: &[u8]) -> Result<Symbol<'lib, T>> {
        self.0.get(symbol).map(|from| {
            Symbol {
                inner: from,
                pd: marker::PhantomData
            }
        })
    }
}

impl fmt::Debug for Library {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Symbol from a library.
///
/// This type is a safeguard against using dynamically loaded symbols after a `Library` is
/// unloaded. Primary method to create an instance of a `Symbol` is via `Library::get`.
///
/// Due to implementation of the `Deref` trait, an instance of `Symbol` may be used as if it was a
/// function or variable directly, without taking care to “extract” function or variable manually
/// most of the time.
///
/// # Examples
///
/// ```no_run
/// # use ::libloading::{ Library, Symbol };
/// # let lib = Library::new("libm.so.6").unwrap();
/// let sin: Symbol<unsafe extern fn(f64) -> f64> = unsafe {
///     lib.get(b"sin\0").unwrap()
/// };
///
/// let sine0 = unsafe { sin(0f64) };
/// assert!(sine0 < 0.1E-10);
/// ```
#[derive(Clone)]
pub struct Symbol<'lib, T: 'lib> {
    inner: imp::Symbol<T>,
    pd: marker::PhantomData<&'lib T>
}

impl<'lib, T> ::std::ops::Deref for Symbol<'lib, T> {
    type Target = T;
    fn deref(&self) -> &T {
        ::std::ops::Deref::deref(&self.inner)
    }
}

impl<'lib, T> fmt::Debug for Symbol<'lib, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}
