//! A memory-safer wrapper around system dynamic library primitives.
//!
//! With this library you can load [dynamic libraries](struct.Library.html) and retrieve
//! [symbols](struct.Symbol.html). This library ensures you won’t somehow close the library before
//! you’re done using the symbol.
//!
//! Less safe platform specific bindings are available in the [`os::platform`](os/index.html)
//! modules.
#![cfg_attr(unix, feature(static_mutex))]
#![cfg_attr(windows, feature(const_fn))]

use std::ffi::{CStr, OsStr};
use std::marker;

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
    /// will not work.
    ///
    /// # Unsafety
    ///
    /// The pointer to a symbol of arbitrary type or kind is returned. Requesting for function
    /// pointer while the symbol is not one and vice versa is not memory safe.
    ///
    /// # Examples
    ///
    /// Simple function:
    ///
    /// ```ignore
    /// let sin: Symbol<extern fn(f64) -> f64> = unsafe {
    ///     lib.get(&CString::new("sin").unwrap()).unwrap()
    /// };
    /// ```
    ///
    /// A static/TLS variable:
    ///
    /// ```ignore
    /// let errno: Symbol<*mut u32> = unsafe {
    ///     lib.get(&CString::new("errno").unwrap()).unwrap()
    /// };
    /// ```
    /// ```
    pub unsafe fn get<'lib, T>(&'lib self, symbol: &CStr) -> Result<Symbol<'lib, T>> {
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

#[cfg(not(target_os="windows"))]
#[test]
fn libm() {
    let lib = Library::new("libm.so.6").unwrap();
    let sin: Symbol<extern fn(f64) -> f64> = unsafe {
        lib.get(&::std::ffi::CString::new("sin").unwrap()).unwrap()
    };
    assert!(sin(::std::f64::INFINITY).is_nan());
    let errno: Symbol<*mut u32> = unsafe {
        lib.get(&::std::ffi::CString::new("errno").unwrap()).unwrap()
    };
    assert!(unsafe { **errno } != 0);
    unsafe { **errno = 0; }
}
