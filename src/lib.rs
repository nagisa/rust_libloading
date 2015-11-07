//! A memory-safer wrapper around system dynamic library primitives.
//!
//! With this library you can load [dynamic libraries](struct.Library.html) and retrieve
//! [symbols](struct.Symbol.html). This library ensures you won’t somehow close the library before
//! you’re done using the symbol.
//!
//! Less safe platform specific bindings are available in the [`os::platform`](os/index.html)
//! modules.
use std::ffi::OsStr;
use std::marker;

#[macro_use]
extern crate lazy_static;

#[cfg(any(target_os="linux",
          target_os="macos",
          target_os="freebsd",
          target_os="dragonfly",
          target_os="bitrig",
          target_os="netbsd",
          target_os="openbsd"))]
use self::os::unix as imp;
#[cfg(target_os="windows")]
use self::os::windows as imp;

pub mod os;
mod util;

pub type Result<T> = ::std::result::Result<T, String>;

/// A dynamically loaded library.
///
/// Only the behaviour that can be reasonably consistently implemented across platforms is provided
/// here.
///
/// # Examples
///
/// ```ignore
/// let lib = Library::new("libm.so.6").unwrap();
/// let ceil: Symbol<extern fn(f64) -> f64> = unsafe {
///     lib.get(&CString::new("ceil").unwrap()).unwrap()
/// };
/// assert_eq!(ceil(0.4), 1.0);
/// ```
pub struct Library(imp::Library);

impl Library {
    /// Find and load a shared library (module).
    ///
    /// Locations where library is searched for is platform specific and can’t be adjusted
    /// portably.
    pub fn new<P: AsRef<OsStr>>(filename: P) -> Result<Library> {
        imp::Library::new(filename).map(Library)
    }

    /// Get a symbol by name.
    ///
    /// Mangling or symbol rustification is not done: trying to `get` something like `x::y`
    /// will not work. Symbol may or may not be terminated with a null byte (see “Premature
    /// optimisation”).
    ///
    /// # Unsafety
    ///
    /// The pointer to a symbol of arbitrary type or kind is returned. Requesting for function
    /// pointer while the symbol is not one and vice versa is not memory safe.
    ///
    /// # Premature optimisation
    ///
    /// You may append a null byte at the end of the byte string to avoid string allocation in some
    /// cases. E.g. for symbol `sin` you may write `b"sin\0"` instead of `b"sin"`.
    ///
    /// # Examples
    ///
    /// Simple function:
    ///
    /// ```ignore
    /// let sin: Symbol<extern fn(f64) -> f64> = unsafe {
    ///     lib.get(b"sin\0").unwrap()
    /// };
    /// ```
    ///
    /// A static or TLS variable:
    ///
    /// ```ignore
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

/// Symbol from a library.
///
/// This type is a safeguard against using dynamically loaded symbols after `Library` is dropped.
/// Primary way to create an instance of a `Symbol` is via `Library::get`.
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

#[cfg(not(any(target_os="windows", target_os="macos")))]
#[test]
fn libm() {
    let lib = Library::new("libm.so.6").unwrap();
    let sin: Symbol<extern fn(f64) -> f64> = unsafe {
        lib.get(b"sin").unwrap()
    };
    assert!(sin(::std::f64::INFINITY).is_nan());
    let errno: Symbol<*mut u32> = unsafe {
        lib.get(b"errno").unwrap()
    };
    assert!(unsafe { **errno } != 0);
    unsafe { **errno = 0; }
}
