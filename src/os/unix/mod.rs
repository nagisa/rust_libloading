/// UNIX implementation of dynamic library loading.
///
/// This module should be expanded with more UNIX-specific functionality in the future.

use std::ffi::{CStr, CString, OsStr, OsString};
use std::marker;
use std::path::PathBuf;
use std::sync::{StaticMutex, MUTEX_INIT};
use std::os::raw;
use std::ptr;
use std::os::unix::ffi::OsStrExt;

// libdl is retarded.
//
// First of all, whole error handling scheme in libdl is done via setting and querying some global
// state, therefore it is not safe to use libdl in MT-capable environment at all. Only in POSIX
// 2008+TC1 a thread-local state was allowed, which for our purposes is not relevant at all.
fn with_dlerror<T, F>(closure: F) -> Result<T, Option<String>>
where F: FnOnce() -> Option<T> {
    // We will guard all uses of libdl library with our own mutex. This makes libdl
    // safe to use in MT programs provided the only way a program uses libdl is via this library.
    static MUTEX: StaticMutex = MUTEX_INIT;
    let _lock = MUTEX.lock();
    // While we could could call libdl here to clear the previous error value, only the dlsym
    // depends on it being cleared beforehand and only in some cases too. We will instead clear the
    // error inside the dlsym binding instead.
    //
    // In all the other cases, clearing the error here will only be hiding misuse of these bindings
    // or the libdl instead.
    closure().map(Ok).unwrap_or_else(|| unsafe {
        // This code will only get executed if the `closure` returns `None`.
        let error = dlerror();
        if error.is_null() {
            // In non-dlsym case this may happen when there’s bugs in our bindings or there’s
            // libloading user of libdl; possibly in another thread.
            return Err(None)
        }
        // You can’t even rely on error string being static here; call to subsequent dlerror may
        // invalidate or overwrite the error message. Why couldn’t they simply give up the
        // ownership over the message?
        // TODO: should do locale-aware conversion here. OTOH Rust doesn’t seem to work well
        // in any system that uses non-utf8 locale, so I doubt there’s a problem here.
        Err(Some(CStr::from_ptr(error).to_string_lossy().into_owned()))
        // FIXME?: Since we do a copy of the error string above, maybe we should call dlerror again
        // to let libdl know it may free the string now?
    })
}


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

    #[inline]
    pub fn new<P: AsRef<OsStr>>(filename: P) -> ::Result<Library> {
        Library::open(Some(filename), RTLD_LAZY)
    }

    #[inline]
    pub fn this() -> Library {
        Library::open(None::<&OsStr>, RTLD_NOW).unwrap()
    }

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


#[link(name = "dl")]
extern {
    fn dlopen(filename: *const raw::c_char, flags: raw::c_int) -> *mut raw::c_void;
    fn dlclose(handle: *mut raw::c_void) -> raw::c_int;
    fn dlsym(handle: *mut raw::c_void, symbol: *const raw::c_char) -> *mut raw::c_void;
    fn dlerror() -> *mut raw::c_char;
}

const RTLD_LAZY: raw::c_int = 1;
const RTLD_NOW: raw::c_int = 2;

#[cfg(target_os="macos")]
pub fn from_library_name<P: AsRef<OsStr>>(name: P) -> PathBuf {
    let mut buffer = OsString::new();
    buffer.push("lib");
    buffer.push(name);
    buffer.push(".dylib");
    buffer.into()
}

#[cfg(any(target_os="linux",
          target_os="freebsd",
          target_os="dragonfly",
          target_os="bitrig",
          target_os="netbsd",
          target_os="openbsd"))]
pub fn from_library_name<P: AsRef<OsStr>>(name: P) -> PathBuf {
    let mut buffer = OsString::new();
    buffer.push("lib");
    buffer.push(name);
    buffer.push(".so");
    buffer.into()
}

#[test]
fn this() {
    Library::this();
}

#[test]
fn new_libm() {
    Library::new(from_library_name("m")).unwrap();
}

#[test]
fn new_m() {
    Library::new("m.so").err().unwrap();
}

#[test]
fn libm_ceil() {
    let lib = Library::new(from_library_name("m")).unwrap();
    let ceil: Symbol<extern fn(f64) -> f64> = unsafe {
        lib.get(&CString::new("ceil").unwrap()).unwrap()
    };
    assert_eq!(ceil(0.45), 1.0);
}
