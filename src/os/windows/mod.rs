/// Windows implementation of dynamic library loading.
///
/// This module should be expanded with more Windows-specific functionality in the future.

extern crate winapi;
extern crate kernel32;
extern crate psapi;

use std::ffi::{CStr, OsStr, OsString};
use std::marker;
use std::mem;
use std::path::PathBuf;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::os::windows::ffi::OsStrExt;


struct Module(pub winapi::HMODULE);

impl Drop for Module {
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


enum LibraryInner {
    New(Module),
    This
}

pub struct Library(LibraryInner);

impl Library {
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
                Some(Library(LibraryInner::New(Module(handle))))
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
        Library(LibraryInner::This)
    }

    pub unsafe fn get<T>(&self, symbol: &CStr) -> ::Result<Symbol<T>> {
        match self.0 {
            LibraryInner::New(ref m) => Library::get_regular(m.0, symbol),
            LibraryInner::This => Library::get_this(symbol)
        }
    }

    unsafe fn get_regular<T>(handle: winapi::HMODULE, symbol: &CStr) -> ::Result<Symbol<T>> {
        with_get_last_error(|| {
            let symbol = kernel32::GetProcAddress(handle, symbol.as_ptr());
            if symbol.is_null() {
                None
            } else {
                Some(Symbol {
                    pointer: symbol,
                    _module: None,
                    pd: marker::PhantomData
                })
            }
        }).map_err(|e| e.unwrap_or_else(||
            panic!("GetProcAddress failed but GetLastError did not report the error")
        ))
    }

    unsafe fn get_this<T>(symbol: &CStr) -> ::Result<Symbol<T>> {
        // We emulate the behaviour of UNIX’s dlopen(NULL) here.
        with_get_last_error(|| {
            let mut modules: [winapi::HMODULE; 2048] = mem::uninitialized();
            let mut count: winapi::DWORD = 0;
            if psapi::EnumProcessModules(kernel32::GetCurrentProcess(), modules.as_mut_ptr(),
                                         2048, &mut count) == 0 { return None }
            for module in &modules[..(count as usize)] {
                let symbol = kernel32::GetProcAddress(*module, symbol.as_ptr());
                if !symbol.is_null() {
                    // If we get here, we found a module, duplicate it.
                    let mut filename: [u16; 2048] = mem::uninitialized();
                    if kernel32::GetModuleFileNameW(*module, filename.as_mut_ptr(), 2047) == 0 {
                        return None
                    }
                    let module = kernel32::LoadLibraryW(filename.as_ptr());
                    return if module.is_null() { None } else { Some(module) };
                }
            }
            None
        }).map_err(|e| e.unwrap_or_else(||
            panic!("GetLastError did not report the error encountered during module search")
        )).and_then(|module| {
            let new_module = Module(module);
            Ok(Symbol {
                _module: Some(new_module),
                ..try!(Library::get_regular(module, symbol))
            })
        })
    }
}


pub struct Symbol<T> {
    pointer: winapi::FARPROC,
    _module: Option<Module>,
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


static USE_THREADERRORMODE: AtomicBool = AtomicBool::new(true);
struct ErrorModeGuard(winapi::DWORD);

impl ErrorModeGuard {
    fn new() -> ErrorModeGuard {
        let mut ret = ErrorModeGuard(0);

        if USE_THREADERRORMODE.load(Ordering::Acquire) {
            if unsafe { kernel32::SetThreadErrorMode(1, &mut ret.0) == 0
                        && kernel32::GetLastError() == winapi::ERROR_CALL_NOT_IMPLEMENTED } {
                USE_THREADERRORMODE.store(false, Ordering::Release);
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
            if USE_THREADERRORMODE.load(Ordering::Relaxed) {
                kernel32::SetThreadErrorMode(self.0, ptr::null_mut());
            } else {
                kernel32::SetErrorMode(self.0);
            }
        }
    }
}


fn with_get_last_error<T, F>(closure: F) -> Result<T, Option<String>>
where F: FnOnce() -> Option<T> {
    closure().map(Ok).unwrap_or_else(|| {
        let error = unsafe { kernel32::GetLastError() };
        Err(if error == 0 {
            None
        } else {
            // TODO: Possibly use FormatMessage (lots of work here)
            Some(format!("Error {}", error))
        })
    })
}


pub fn from_library_name<P: AsRef<OsStr>>(name: P) -> PathBuf {
    let mut buffer = OsString::new();
    buffer.push(name);
    buffer.push(".dll");
    buffer.into()
}

#[test]
fn works_this() {
    let this = Library::this();
    let ceil: Symbol<extern fn(f64) -> f64> = unsafe {
        this.get(&::std::ffi::CString::new("ceil").unwrap()).unwrap()
    };
    assert_eq!(ceil(0.45), 1.0);
}

#[test]
fn works_new_kernel32() {
    let that = Library::new(from_library_name("kernel32")).unwrap();
    unsafe {
        that.get::<*mut usize>(&::std::ffi::CString::new("GetLastError").unwrap()).unwrap();
    }
}

#[test]
fn fails_new_kernel23() {
    Library::new("kernel23").err().unwrap();
}
