/// Windows implementation of dynamic library loading.
///
/// This module should be expanded with more Windows-specific functionality in the future.
extern crate winapi;
extern crate kernel32;

use util::{CowCString, CStringAsRef};

use std::ffi::{OsStr, OsString};
use std::marker;
use std::mem;
use std::ptr;
use std::sync::atomic::{AtomicBool, ATOMIC_BOOL_INIT, Ordering};
use std::os::windows::ffi::{OsStrExt, OsStringExt};


/// A platform-specific equivalent of the cross-platform `Library`.
pub struct Library(winapi::HMODULE);

impl Library {
    /// Find and load a shared library (module).
    ///
    /// Locations where library is searched for is platform specific and can’t be adjusted
    /// portably.
    ///
    /// Corresponds to `LoadLibraryW(filename)`.
    #[inline]
    pub fn new<P: AsRef<OsStr>>(filename: P) -> ::Result<Library> {
        let mut wide_filename: Vec<u16> = filename.as_ref().encode_wide().collect();
        wide_filename.push(0);
        wide_filename.shrink_to_fit();
        let _guard = ErrorModeGuard::new();

        let ret = with_get_last_error(|| {
            // Make sure no winapi calls as a result of drop happen inside this closure, because
            // otherwise that might change the return value of the GetLastError.
            let handle = unsafe { kernel32::LoadLibraryW(wide_filename.as_ptr()) };
            if handle.is_null()  {
                None
            } else {
                Some(Library(handle))
            }
        }).map_err(|e| e.unwrap_or_else(||
            panic!("LoadLibraryW failed but GetLastError did not report the error")
        ));

        drop(wide_filename); // Drop wide_filename here to ensure it doesn’t get moved and dropped
                             // inside the closure by mistake.
        ret
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
    pub unsafe fn get<T>(&self, symbol: &[u8]) -> ::Result<Symbol<T>> {
        let symbol = try!(CowCString::from_bytes(symbol));
        with_get_last_error(|| {
            let symbol = kernel32::GetProcAddress(self.0, symbol.cstring_ref());
            if symbol.is_null() {
                None
            } else {
                Some(Symbol {
                    pointer: symbol,
                    pd: marker::PhantomData
                })
            }
        }).map_err(|e| e.unwrap_or_else(||
            panic!("GetProcAddress failed but GetLastError did not report the error")
        ))
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        with_get_last_error(|| {
            if unsafe { kernel32::FreeLibrary(self.0) == 0 } {
                None
            } else {
                Some(())
            }
        }).unwrap()
    }
}

impl ::std::fmt::Debug for Library {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        unsafe {
            let mut buf: [winapi::WCHAR; 1024] = mem::uninitialized();
            let len = kernel32::GetModuleFileNameW(self.0,
                                                   (&mut buf[..]).as_mut_ptr(), 1024) as usize;
            if len == 0 {
                f.write_str(&format!("Library@{:p}", self.0))
            } else {
                let string: OsString = OsString::from_wide(&buf[..len]);
                f.write_str(&format!("Library@{:p} from {:?}", self.0, string))
            }
        }
    }
}

/// Symbol from a library.
///
/// A major difference compared to the cross-platform `Symbol` is that this does not ensure the
/// `Symbol` does not outlive `Library` it comes from.
pub struct Symbol<T> {
    pointer: winapi::FARPROC,
    pd: marker::PhantomData<T>
}

impl<T> ::std::ops::Deref for Symbol<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe {
            // Additional reference level for a dereference on `deref` return value.
            ::std::mem::transmute(&self.pointer)
        }
    }
}

impl<T> ::std::fmt::Debug for Symbol<T> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        f.write_str(&format!("Symbol@{:p}", self.pointer))
    }
}


static USE_ERRORMODE: AtomicBool = ATOMIC_BOOL_INIT;
struct ErrorModeGuard(winapi::DWORD);

impl ErrorModeGuard {
    fn new() -> ErrorModeGuard {
        let mut ret = ErrorModeGuard(0);

        if !USE_ERRORMODE.load(Ordering::Acquire) {
            if unsafe { kernel32::SetThreadErrorMode(1, &mut ret.0) == 0
                        && kernel32::GetLastError() == winapi::ERROR_CALL_NOT_IMPLEMENTED } {
                USE_ERRORMODE.store(true, Ordering::Release);
            } else {
                return ret;
            }
        }
        ret.0 = unsafe { kernel32::SetErrorMode(1) };
        ret
    }
}

impl Drop for ErrorModeGuard {
    fn drop(&mut self) {
        unsafe {
            if !USE_ERRORMODE.load(Ordering::Relaxed) {
                kernel32::SetThreadErrorMode(self.0, ptr::null_mut());
            } else {
                kernel32::SetErrorMode(self.0);
            }
        }
    }
}

fn with_get_last_error<T, F>(closure: F) -> Result<T, Option<::std::io::Error>>
where F: FnOnce() -> Option<T> {
    closure().ok_or_else(|| {
        let error = unsafe { kernel32::GetLastError() };
        if error == 0 {
            None
        } else {
            Some(::std::io::Error::from_raw_os_error(error as i32))
        }
    })
}

#[test]
fn works_GetLastError() {
    let that = Library::new("kernel32.dll").unwrap();
    unsafe {
        that.get::<*mut usize>(b"GetLastError").unwrap();
    }
}

#[test]
fn works_GetLastError0() {
    let that = Library::new("kernel32.dll").unwrap();
    unsafe {
        that.get::<*mut usize>(b"GetLastError\0").unwrap();
    }
}

#[test]
fn fails_new_kernel23() {
    Library::new("kernel23").err().unwrap();
}
