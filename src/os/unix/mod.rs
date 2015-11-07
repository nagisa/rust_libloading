/// UNIX implementation of dynamic library loading.
///
/// This module should eventually be expanded with more UNIX-specific functionality in the future.

use std::ffi::{CStr, CString, OsStr};
use std::marker;
use std::sync::Mutex;
use std::os::raw;
use std::ptr;
use std::mem;
use std::os::unix::ffi::OsStrExt;

// libdl is retarded.
//
// First of all, whole error handling scheme in libdl is done via setting and querying some global
// state, therefore it is not safe to use libdl in MT-capable environment at all. Only in POSIX
// 2008+TC1 a thread-local state was allowed, which for our purposes is way too late.
fn with_dlerror<T, F>(closure: F) -> Result<T, Option<String>>
where F: FnOnce() -> Option<T> {
    // We will guard all uses of libdl library with our own mutex. This makes libdl
    // safe to use in MT programs provided the only way a program uses libdl is via this library.
    lazy_static! {
        static ref MUTEX: Mutex<()> = Mutex::new(());
    }
    let _lock = MUTEX.lock();
    // While we could could call libdl here to clear the previous error value, only the dlsym
    // depends on it being cleared beforehand and only in some cases too. We will instead clear the
    // error inside the dlsym binding instead.
    //
    // In all the other cases, clearing the error here will only be hiding misuse of these bindings
    // or the libdl.
    closure().ok_or_else(|| unsafe {
        // This code will only get executed if the `closure` returns `None`.
        let error = dlerror();
        if error.is_null() {
            // In non-dlsym case this may happen when there’s bugs in our bindings or there’s
            // non-libloading user of libdl; possibly in another thread.
            None
        } else {
            // You can’t even rely on error string being static here; call to subsequent dlerror
            // may invalidate or overwrite the error message. Why couldn’t they simply give up the
            // ownership over the message?
            // TODO: should do locale-aware conversion here. OTOH Rust doesn’t seem to work well in
            // any system that uses non-utf8 locale, so I doubt there’s a problem here.
            Some(CStr::from_ptr(error).to_string_lossy().into_owned())
            // FIXME?: Since we do a copy of the error string above, maybe we should call dlerror
            // again to let libdl know it may free its copy of the string now?
        }
    })
}

/// A platform-specific equivalent of the cross-platform `Library`.
pub struct Library {
    handle: *mut raw::c_void
}

impl Library {
    fn open<P>(filename: Option<P>, flags: raw::c_int) -> ::Result<Library>
    where P: AsRef<OsStr> {
        let filename = match filename {
            None => Ok(None),
            Some(f) => CString::new(f.as_ref().as_bytes()).map(Some),
        };
        let ptr = match filename {
            Err(_) => return Err("library name contains null bytes".into()),
            Ok(None) => ptr::null(),
            Ok(Some(f)) => f.as_ptr(),
        };
        with_dlerror(|| {
            let result = unsafe { dlopen(ptr, flags) };
            if result.is_null() {
                None
            } else {
                Some(Library {
                    handle: result
                })
            }
        }).map_err(|e| e.unwrap_or_else(||
            panic!("dlopen failed but dlerror did not report anything")
        ))
    }

    /// Find and load a shared library (module).
    ///
    /// Locations where library is searched for is platform specific and can’t be adjusted
    /// portably.
    ///
    /// Corresponds to `dlopen(filename)`.
    #[inline]
    pub fn new<P: AsRef<OsStr>>(filename: P) -> ::Result<Library> {
        Library::open(Some(filename), RTLD_LAZY)
    }

    /// Load the dynamic libraries linked into main program.
    ///
    /// This allows retrieving symbols from any **dynamic** library linked into the program,
    /// without specifying the exact library.
    ///
    /// Corresponds to `dlopen(NULL)`.
    #[inline]
    pub fn this() -> Library {
        Library::open(None::<&OsStr>, RTLD_NOW).unwrap()
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
    /// The return value does not ensure the symbol does not outlive the library.
    pub unsafe fn get<T>(&self, symbol: &CStr) -> ::Result<Symbol<T>> {
        // `dlsym` may return nullptr in two cases: when a symbol genuinely points to a null
        // pointer or the symbol cannot be found. In order to detect this case a double dlerror
        // pattern must be used, which is, sadly, a little bit racy.
        //
        // We try to leave as little space as possible for this to occur, but we can’t exactly
        // fully prevent it.
        match with_dlerror(|| {
            dlerror();
            let symbol = dlsym(self.handle, symbol.as_ptr());
            if symbol.is_null() {
                None
            } else {
                Some(Symbol {
                    pointer: symbol,
                    pd: marker::PhantomData
                })
            }
        }) {
            Err(None) => Ok(Symbol {
                pointer: ptr::null_mut(),
                pd: marker::PhantomData
            }),
            Err(e) => Err(e.unwrap()),
            Ok(x) => Ok(x)
        }
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        with_dlerror(|| if unsafe { dlclose(self.handle) } == 0 {
            Some(())
        } else {
            None
        }).unwrap();
    }
}

impl ::std::fmt::Debug for Library {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        f.write_str(&format!("Library@{:p}", self.handle))
    }
}

/// Symbol from a library.
///
/// A major difference compared to the cross-platform `Symbol` is that this does not ensure the
/// `Symbol` does not outlive `Library` it comes from.
pub struct Symbol<T> {
    pointer: *mut raw::c_void,
    pd: marker::PhantomData<T>
}

impl<T> ::std::ops::Deref for Symbol<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe {
            &*(&self.pointer as *const _ as *const T)
        }
    }
}

impl<T> ::std::fmt::Debug for Symbol<T> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        unsafe {
            let mut info: DlInfo = mem::uninitialized();
            if dladdr(self.pointer, &mut info) != 0 {
                if info.dli_sname.is_null() {
                    f.write_str(&format!("Symbol@{:p} from {:?}",
                                         self.pointer,
                                         CStr::from_ptr(info.dli_fname)))
                } else {
                    f.write_str(&format!("Symbol {:?}@{:p} from {:?}",
                                         CStr::from_ptr(info.dli_sname), self.pointer,
                                         CStr::from_ptr(info.dli_fname)))
                }
            } else {
                f.write_str(&format!("Symbol@{:p}", self.pointer))
            }
        }
    }
}

// Platform specific things

extern {
    fn dlopen(filename: *const raw::c_char, flags: raw::c_int) -> *mut raw::c_void;
    fn dlclose(handle: *mut raw::c_void) -> raw::c_int;
    fn dlsym(handle: *mut raw::c_void, symbol: *const raw::c_char) -> *mut raw::c_void;
    fn dlerror() -> *mut raw::c_char;
    fn dladdr(addr: *mut raw::c_void, info: *mut DlInfo) -> raw::c_int;
}

const RTLD_LAZY: raw::c_int = 1;
const RTLD_NOW: raw::c_int = 2;

#[repr(C)]
struct DlInfo {
  dli_fname: *const raw::c_char,
  dli_fbase: *mut raw::c_void,
  dli_sname: *const raw::c_char,
  dli_saddr: *mut raw::c_void
}

#[test]
fn this() {
    Library::this();
}

#[cfg(all(test,
          any(target_os="linux",
          target_os="freebsd",
          target_os="dragonfly",
          target_os="bitrig",
          target_os="netbsd",
          target_os="openbsd")))]
const LIBM: &'static str = "libm.so.6";

#[cfg(all(test, target_os="macos"))]
const LIBM: &'static str = "libm.dylib";

#[test]
fn new_libm() {
    Library::new(LIBM).unwrap();
}

#[test]
fn new_m() {
    Library::new("m").err().unwrap();
}

#[test]
fn libm_ceil() {
    let lib = Library::new(LIBM).unwrap();
    let ceil: Symbol<extern fn(f64) -> f64> = unsafe {
        lib.get(&CString::new("ceil").unwrap()).unwrap()
    };
    assert_eq!(ceil(0.45), 1.0);
}
