/// Windows implementation of dynamic library loading.
///
/// This module should be expanded with more Windows-specific functionality in the future.

extern crate winapi;
extern crate kernel32;

use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use winapi::{HMODULE, FARPROC, DWORD, ERROR_CALL_NOT_IMPLEMENTED};

pub struct Library {
    handle: HMODULE
}

unsafe fn with_get_last_error<T, F>(closure: F) -> Result<T, Option<String>>
where F: FnOnce() -> Option<T> {
    closure.map(Ok).unwrap_or_else(|| {
        let error = kernel32::GetLastError();
        Err(if error == 0 {
            None
        } else {
            // TODO: Possibly use FormatMessage (lots of work here)
            Some(format!("Error {}", error))
        })
    })
}

impl Library {
    #[inline]
    pub fn new<P: AsRef<OsStr>>(filename: P) -> ::Result<Library> {
        let wide_filename: Vec<u16> = filename.as_ref().encode_wide().collect();
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
                Some(Library {
                    handle: handle
                })
            }
        }).map_err(|e| e.unwrap_or_else(||
            panic!("LoadLibraryW failed but GetLastError did not report the error")
        ));

        drop(wide_filename); // Drop wide_filename here to ensure it doesn’t get moved and dropped
                             // inside the closure by mistake.
        ret
    }

    #[inline]
    pub fn this() -> Library {
        with_get_last_error(|| {
            let mut ret = Library {
                handle: ptr::null_mut()
            };
            if unsafe { kernel32::GetModuleHandleExW(0, ptr::null(), &mut (ret.0)) == 0 } {
                None
            } else {
                Some(ret)
            }
        }).expect("GetModuleHandleExW failed, but it shouldn’t")
    }

    pub unsafe fn get(&self, symbol: &CStr) -> ::Result<FARPROC> {
        with_get_last_error(|| {
            let symbol = unsafe { GetProcAddress(self.0, symbol.as_ptr()) };
            if symbol.is_null() {
                None
            } else {
                Some(symbol)
            }
        }).map_err(|e| e.unwrap_or_else(||
            panic!("GetProcAddress failed but GetLastError did not report the error")
        ))
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        with_error(|| {
            if unsafe { kernel32::FreeLibrary(self.handle) == 0 } {
                None
            } else {
                Some(())
            }
        }).unwrap()
    }
}


static USE_THREADERRORMODE: AtomicBool = AtomicBool::new(true);

struct ErrorModeGuard(DWORD);

impl ErrorModeGuard {
    fn new() -> ErrorModeGuard {
        let mut ret = ErrorModeGuard(0);

        if USE_THREADERRORMODE.load(Ordering::Acquire) {
            if unsafe { kernel32::SetThreadErrorMode(1, &mut ret.0) == 0
                        && kernel32::GetLastError() == ERROR_CALL_NOT_IMPLEMENTED } {
                USE_THREADERRORMODE.store(false, Ordering::Store)
            } else {
                return ret;
            }
        }
        ret.0 = unsafe { SetErrorMode(1) };
        ret
    }
}

impl Drop for ErrorModeGuard {
    fn drop(&mut self) {
        unsafe {
            if USE_THREADERRORMODE.load(Ordering::Relaxed) {
                kernel32::SetThreadErrorMode(self.0, ptr::null_mut());
            } else {
                kernel32::SetErrorMode(self.0);
            }
        }
    }
}

pub fn from_library_name(name: &OsStr) -> PathBuf {
    let buffer = OsString::new();
    buffer.push(name);
    buffer.push(".dll");
    buffer.into()
}
