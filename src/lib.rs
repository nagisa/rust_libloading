#![cfg_attr(unix, feature(cstr_to_str, static_mutex))]
#![cfg_attr(windows, feature(const_fn, result_expect))]

use std::ffi::{CStr, OsStr};
use std::marker;
use std::path::PathBuf;

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
/// See the `os::platform` modules for platform-specific and less safe implementations.
///
/// # Examples
///
/// ```ignore
/// let lib = Library::new(from_library_name("m")).unwrap();
/// let ceil: Symbol<extern fn(f64) -> f64> = unsafe {
///     lib.get(&CString::new("ceil").unwrap()).unwrap()
/// };
/// assert_eq!(ceil(0.4), 1.0);
/// ```
pub struct Library(imp::Library);

impl Library {
    /// Find and load a shared library.
    ///
    /// Locations where library is searched for is platform specific and can’t be changed portably.
    pub fn new<P: AsRef<OsStr>>(filename: P) -> Result<Library> {
        imp::Library::new(filename).map(Library)
    }

    /// Load all already loaded libraries.
    ///
    /// This allows retrieving symbols from already **dynamically** loaded libraries.
    /// “Dynamically” here is a very important part: you cannot load symbols linked into the
    /// executable statically. For example following code will not work:
    ///
    /// ```ignore
    /// let lib = Library::this();
    /// let main: Symbol<extern fn() -> usize> = unsafe {
    ///     // function `main` is usually linked-in statically
    ///     lib.get(&::std::ffi::CString::new("main").unwrap()).unwrap()
    /// };
    /// ```
    pub fn this() -> Library {
        Library(imp::Library::this())
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
    /// ```ignore
    /// // Function
    /// let sin: Symbol<extern fn(f64) -> f64> = unsafe {
    ///     lib.get(&CString::new("sin").unwrap()).unwrap()
    /// };
    /// // Static/TLS variable
    /// let errno: Symbol<*mut u32> = unsafe {
    ///     lib.get(&CString::new("errno").unwrap()).unwrap()
    /// };
    /// ```
    pub unsafe fn get<'lib, T>(&'lib self, symbol: &CStr) -> Result<Symbol<'lib, T>> {
        self.0.get(symbol).map(|from| {
            Symbol {
                pointer: from as *mut _,
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
    pointer: *mut ::std::os::raw::c_void,
    pd: marker::PhantomData<&'lib T>
}

impl<'lib, T> ::std::ops::Deref for Symbol<'lib, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe {
            &*(&self.pointer as *const _ as *const T)
        }
    }
}

/// Convert library name into a filename one can pass to `Library::new()`
///
/// This function converts library name into a filename the library is expected to have on target
/// platform:
///
/// * UNIXes: `m` → `libm.so`;
/// * OS X: `m` → `libm.dylib`;
/// * Windows: `m` → `m.dll`;
///
/// Allowing for somewhat more platform independent code.
///
/// # Examples
///
/// ```ignore
/// let math = Library::new(from_library_name("m")).unwrap();
/// // use math library here
/// ```
pub fn from_library_name<P: AsRef<OsStr>>(name: P) -> PathBuf {
    imp::from_library_name(name)
}

#[cfg(not(target_os="windows"))]
#[test]
fn libm() {
    let lib = Library::new(from_library_name("m")).unwrap();
    let sin: Symbol<extern fn(f64) -> f64> = unsafe {
        lib.get(&::std::ffi::CString::new("sin").unwrap()).unwrap()
    };
    assert!(sin(::std::f64::INFINITY).is_nan());
    let errno: Symbol<*mut u32> = unsafe {
        lib.get(&::std::ffi::CString::new("errno").unwrap()).unwrap()
    };
    assert!(unsafe { **errno } != 0);
}

#[cfg(not(target_os="windows"))]
#[test]
fn this_strlen() {
    // I couldn’t find anything that works on windows here hehe
    const SYMBOL: &'static str = "strlen";
    let lib = Library::this();
    let _symbol: Symbol<extern fn() -> usize> = unsafe {
        lib.get(&::std::ffi::CString::new(SYMBOL).unwrap()).unwrap()
    };
}
