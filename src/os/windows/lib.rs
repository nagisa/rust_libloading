use kernel32;
use os::util::CowCString;
use os::util::CStringAsRef;
use os::windows::util;
use os::windows::ErrorModeGuard;
use result::Result as R;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;
use winapi::HMODULE;
use winapi::WCHAR;

/// A platform-specific equivalent of the cross-platform `Library`.
pub struct Lib {
    handle: HMODULE
}

impl Lib {
    /// Find and load a shared library (module).
    ///
    /// Locations where library is searched for is platform specific and can’t be adjusted
    /// portably.
    ///
    /// Corresponds to `LoadLibraryW(filename)`.
    pub fn new<P: AsRef<OsStr>>(filename: P) -> R<Lib> {
        let wide_filename: Vec<u16> = filename.as_ref().encode_wide().chain(Some(0)).collect();
        let _guard = ErrorModeGuard::new();

        let ret = util::with_get_last_error(|| {
            // Make sure no winapi calls as a result of drop happen inside this closure, because
            // otherwise that might change the return value of the GetLastError.
            let handle = unsafe { kernel32::LoadLibraryW(wide_filename.as_ptr()) };
            if handle.is_null()  {
                None
            } else {
                let lib = Lib { handle: handle };
                Some(lib)
            }
        }).map_err(|e| e.unwrap_or_else(||
            panic!("LoadLibraryW failed but GetLastError did not report the error")
        ));

        drop(wide_filename); // Drop wide_filename here to ensure it doesn’t get moved and dropped
                             // inside the closure by mistake. See comment inside the closure.
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
    pub unsafe fn get<T>(&self, symbol: &[u8]) -> R<*const T> {
        let symbol = try!(CowCString::from_bytes(symbol));
        util::with_get_last_error(|| {
            let symbol = kernel32::GetProcAddress(self.handle, symbol.cstring_ref());
            if symbol.is_null() {
                None
            } else {
                Some(mem::transmute(symbol))
            }
        }).map_err(|e| e.unwrap_or_else(||
            panic!("GetProcAddress failed but GetLastError did not report the error")
        ))
    }
}

impl Debug for Lib {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        unsafe {
            let mut buf: [WCHAR; 1024] = mem::uninitialized();
            let len = kernel32::GetModuleFileNameW(self.handle,
                                                   (&mut buf[..]).as_mut_ptr(), 1024) as usize;
            if len == 0 {
                f.write_str(&format!("Library@{:p}", self.handle))
            } else {
                let string: OsString = OsString::from_wide(&buf[..len]);
                f.write_str(&format!("Library@{:p} from {:?}", self.handle, string))
            }
        }
    }
}

impl Drop for Lib {
    fn drop(&mut self) {
        util::with_get_last_error(|| {
            if unsafe { kernel32::FreeLibrary(self.handle) == 0 } {
                None
            } else {
                Some(())
            }
        }).unwrap()
    }
}
