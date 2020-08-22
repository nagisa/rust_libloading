// A hack for docs.rs to build documentation that has both windows and linux documentation in the
// same rustdoc build visible.
#[cfg(all(docsrs, not(windows)))]
mod windows_imports {
    pub(super) enum WORD {}
    pub(super) enum DWORD {}
    pub(super) enum HMODULE {}
    pub(super) enum FARPROC {}
}
#[cfg(windows)]
mod windows_imports {
    extern crate winapi;
    pub(super) use self::winapi::shared::minwindef::{WORD, DWORD, HMODULE, FARPROC};
    pub(super) use self::winapi::shared::ntdef::WCHAR;
    pub(super) use self::winapi::shared::winerror;
    pub(super) use self::winapi::um::{errhandlingapi, libloaderapi};
    pub(super) use std::os::windows::ffi::{OsStrExt, OsStringExt};
    pub(super) const SEM_FAILCE: DWORD = 1;
}

use self::windows_imports::*;
use util::{ensure_compatible_types, cstr_cow_from_bytes};

use std::ffi::{OsStr, OsString};
use std::{fmt, io, marker, mem, ptr};
use std::sync::atomic::{AtomicBool, Ordering};

/// A platform-specific counterpart of the cross-platform [`Library`](crate::Library).
pub struct Library(HMODULE);

unsafe impl Send for Library {}
// Now, this is sort-of-tricky. MSDN documentation does not really make any claims as to safety of
// the Win32 APIs. Sadly, whomever I asked, even current and former Microsoft employees, couldn’t
// say for sure, whether the Win32 APIs used to implement `Library` are thread-safe or not.
//
// My investigation ended up with a question about thread-safety properties of the API involved
// being sent to an internal (to MS) general question mailing-list. The conclusion of the mail is
// as such:
//
// * Nobody inside MS (at least out of all the people who have seen the question) knows for
//   sure either;
// * However, the general consensus between MS developers is that one can rely on the API being
//   thread-safe. In case it is not thread-safe it should be considered a bug on the Windows
//   part. (NB: bugs filled at https://connect.microsoft.com/ against Windows Server)
unsafe impl Sync for Library {}

impl Library {
    /// Find and load a module.
    ///
    /// If the `filename` specifies a full path, the function only searches that path for the
    /// module. Otherwise, if the `filename` specifies a relative path or a module name without a
    /// path, the function uses a windows-specific search strategy to find the module; for more
    /// information, see the [Remarks on MSDN][msdn].
    ///
    /// This is equivalent to [`Library::load_with_flags`]`(filename, 0)`.
    ///
    /// [msdn]: https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryw#remarks
    #[inline]
    pub fn new<P: AsRef<OsStr>>(filename: P) -> Result<Library, crate::Error> {
        Library::load_with_flags(filename, 0)
    }

    /// Load the `Library` representing the original program executable.
    ///
    /// Note that behaviour of `Library` loaded with this method is different from
    /// Libraries loaded with [`os::unix::Library::this`]. For more information refer to [MSDN].
    ///
    /// Corresponds to `GetModuleHandleExW(0, NULL, _)`.
    ///
    /// [`os::unix::Library::this`]: crate::os::unix::Library::this
    /// [MSDN]: https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getmodulehandleexw
    pub fn this() -> Result<Library, crate::Error> {
        unsafe {
            let mut handle: HMODULE = std::ptr::null_mut();
            with_get_last_error(|source| crate::Error::GetModuleHandleExW { source }, || {
                let result = libloaderapi::GetModuleHandleExW(0, std::ptr::null_mut(), &mut handle);
                if result == 0 {
                    None
                } else {
                    Some(Library(handle))
                }
            }).map_err(|e| e.unwrap_or(crate::Error::GetModuleHandleExWUnknown))
        }
    }

    /// Find and load a module, additionally adjusting behaviour with flags.
    ///
    /// See [`Library::new`] for documentation on handling of the `filename` argument. See the
    /// [flag table on MSDN][flags] for information on applicable values for the `flags` argument.
    ///
    /// Corresponds to `LoadLibraryExW(filename, reserved: NULL, flags)`.
    ///
    /// [flags]: https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw#parameters
    pub fn load_with_flags<P: AsRef<OsStr>>(filename: P, flags: DWORD) -> Result<Library, crate::Error> {
        let wide_filename: Vec<u16> = filename.as_ref().encode_wide().chain(Some(0)).collect();
        let _guard = ErrorModeGuard::new();

        let ret = with_get_last_error(|source| crate::Error::LoadLibraryW { source }, || {
            // Make sure no winapi calls as a result of drop happen inside this closure, because
            // otherwise that might change the return value of the GetLastError.
            let handle = unsafe {
                libloaderapi::LoadLibraryExW(wide_filename.as_ptr(), std::ptr::null_mut(), flags)
            };
            if handle.is_null()  {
                None
            } else {
                Some(Library(handle))
            }
        }).map_err(|e| e.unwrap_or(crate::Error::LoadLibraryWUnknown));
        drop(wide_filename); // Drop wide_filename here to ensure it doesn’t get moved and dropped
                             // inside the closure by mistake. See comment inside the closure.
        ret
    }

    /// Get a pointer to function or static variable by symbol name.
    ///
    /// The `symbol` may not contain any null bytes, with an exception of last byte. A null
    /// terminated `symbol` may avoid a string allocation in some cases.
    ///
    /// Symbol is interpreted as-is; no mangling is done. This means that symbols like `x::y` are
    /// most likely invalid.
    ///
    /// # Safety
    ///
    /// This function does not validate the type `T`. It is up to the user of this function to
    /// ensure that the loaded symbol is in fact a `T`. Using a value with a wrong type has no
    /// definied behaviour.
    pub unsafe fn get<T>(&self, symbol: &[u8]) -> Result<Symbol<T>, crate::Error> {
        ensure_compatible_types::<T, FARPROC>()?;
        let symbol = cstr_cow_from_bytes(symbol)?;
        with_get_last_error(|source| crate::Error::GetProcAddress { source }, || {
            let symbol = libloaderapi::GetProcAddress(self.0, symbol.as_ptr());
            if symbol.is_null() {
                None
            } else {
                Some(Symbol {
                    pointer: symbol,
                    pd: marker::PhantomData
                })
            }
        }).map_err(|e| e.unwrap_or(crate::Error::GetProcAddressUnknown))
    }

    /// Get a pointer to function or static variable by ordinal number.
    ///
    /// # Safety
    ///
    /// Pointer to a value of arbitrary type is returned. Using a value with wrong type is
    /// undefined.
    pub unsafe fn get_ordinal<T>(&self, ordinal: WORD) -> Result<Symbol<T>, crate::Error> {
        ensure_compatible_types::<T, FARPROC>()?;
        with_get_last_error(|source| crate::Error::GetProcAddress { source }, || {
            let ordinal = ordinal as usize as *mut _;
            let symbol = libloaderapi::GetProcAddress(self.0, ordinal);
            if symbol.is_null() {
                None
            } else {
                Some(Symbol {
                    pointer: symbol,
                    pd: marker::PhantomData
                })
            }
        }).map_err(|e| e.unwrap_or(crate::Error::GetProcAddressUnknown))
    }

    /// Convert the `Library` to a raw handle.
    pub fn into_raw(self) -> HMODULE {
        let handle = self.0;
        mem::forget(self);
        handle
    }

    /// Convert a raw handle to a `Library`.
    ///
    /// # Safety
    ///
    /// The handle shall be a result of a successful call of `LoadLibraryW` or a
    /// handle previously returned by the `Library::into_raw` call.
    pub unsafe fn from_raw(handle: HMODULE) -> Library {
        Library(handle)
    }

    /// Unload the library.
    pub fn close(self) -> Result<(), crate::Error> {
        with_get_last_error(|source| crate::Error::FreeLibrary { source }, || {
            if unsafe { libloaderapi::FreeLibrary(self.0) == 0 } {
                None
            } else {
                Some(())
            }
        }).map_err(|e| e.unwrap_or(crate::Error::FreeLibraryUnknown))
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        unsafe { libloaderapi::FreeLibrary(self.0); }
    }
}

impl fmt::Debug for Library {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            // FIXME: use Maybeuninit::uninit_array when stable
            let mut buf =
                mem::MaybeUninit::<[mem::MaybeUninit::<WCHAR>; 1024]>::uninit().assume_init();
            let len = libloaderapi::GetModuleFileNameW(self.0,
                (&mut buf[..]).as_mut_ptr().cast(), 1024) as usize;
            if len == 0 {
                f.write_str(&format!("Library@{:p}", self.0))
            } else {
                let string: OsString = OsString::from_wide(
                    // FIXME: use Maybeuninit::slice_get_ref when stable
                    &*(&buf[..len] as *const [_] as *const [WCHAR])
                );
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
    pointer: FARPROC,
    pd: marker::PhantomData<T>
}

impl<T> Symbol<T> {
    /// Convert the loaded Symbol into a handle.
    pub fn into_raw(self) -> FARPROC {
        let pointer = self.pointer;
        mem::forget(self);
        pointer
    }
}

impl<T> Symbol<Option<T>> {
    /// Lift Option out of the symbol.
    pub fn lift_option(self) -> Option<Symbol<T>> {
        if self.pointer.is_null() {
            None
        } else {
            Some(Symbol {
                pointer: self.pointer,
                pd: marker::PhantomData,
            })
        }
    }
}

unsafe impl<T: Send> Send for Symbol<T> {}
unsafe impl<T: Sync> Sync for Symbol<T> {}

impl<T> Clone for Symbol<T> {
    fn clone(&self) -> Symbol<T> {
        Symbol { ..*self }
    }
}

impl<T> ::std::ops::Deref for Symbol<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe {
            // Additional reference level for a dereference on `deref` return value.
            &*(&self.pointer as *const *mut _ as *const T)
        }
    }
}

impl<T> fmt::Debug for Symbol<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&format!("Symbol@{:p}", self.pointer))
    }
}

static USE_ERRORMODE: AtomicBool = AtomicBool::new(false);
struct ErrorModeGuard(DWORD);

impl ErrorModeGuard {
    #[allow(clippy::if_same_then_else)]
    fn new() -> Option<ErrorModeGuard> {
        unsafe {
            if !USE_ERRORMODE.load(Ordering::Acquire) {
                let mut previous_mode = 0;
                let success = errhandlingapi::SetThreadErrorMode(SEM_FAILCE, &mut previous_mode) != 0;
                if !success && errhandlingapi::GetLastError() == winerror::ERROR_CALL_NOT_IMPLEMENTED {
                    USE_ERRORMODE.store(true, Ordering::Release);
                } else if !success {
                    // SetThreadErrorMode failed with some other error? How in the world is it
                    // possible for what is essentially a simple variable swap to fail?
                    // For now we just ignore the error -- the worst that can happen here is
                    // the previous mode staying on and user seeing a dialog error on older Windows
                    // machines.
                    return None;
                } else if previous_mode == SEM_FAILCE {
                    return None;
                } else {
                    return Some(ErrorModeGuard(previous_mode));
                }
            }
            match errhandlingapi::SetErrorMode(SEM_FAILCE) {
                SEM_FAILCE => {
                    // This is important to reduce racy-ness when this library is used on multiple
                    // threads. In particular this helps with following race condition:
                    //
                    // T1: SetErrorMode(SEM_FAILCE)
                    // T2: SetErrorMode(SEM_FAILCE)
                    // T1: SetErrorMode(old_mode) # not SEM_FAILCE
                    // T2: SetErrorMode(SEM_FAILCE) # restores to SEM_FAILCE on drop
                    //
                    // This is still somewhat racy in a sense that T1 might restore the error
                    // mode before T2 finishes loading the library, but that is less of a
                    // concern – it will only end up in end user seeing a dialog.
                    //
                    // Also, SetErrorMode itself is probably not an atomic operation.
                    None
                }
                a => Some(ErrorModeGuard(a))
            }
        }
    }
}

impl Drop for ErrorModeGuard {
    fn drop(&mut self) {
        unsafe {
            if !USE_ERRORMODE.load(Ordering::Relaxed) {
                errhandlingapi::SetThreadErrorMode(self.0, ptr::null_mut());
            } else {
                errhandlingapi::SetErrorMode(self.0);
            }
        }
    }
}

fn with_get_last_error<T, F>(wrap: fn(crate::error::WindowsError) -> crate::Error, closure: F)
-> Result<T, Option<crate::Error>>
where F: FnOnce() -> Option<T> {
    closure().ok_or_else(|| {
        let error = unsafe { errhandlingapi::GetLastError() };
        if error == 0 {
            None
        } else {
            Some(wrap(crate::error::WindowsError(io::Error::from_raw_os_error(error as i32))))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn works_getlasterror() {
        let lib = Library::new("kernel32.dll").unwrap();
        let gle: Symbol<unsafe extern "system" fn() -> DWORD> = unsafe {
            lib.get(b"GetLastError").unwrap()
        };
        unsafe {
            errhandlingapi::SetLastError(42);
            assert_eq!(errhandlingapi::GetLastError(), gle())
        }
    }

    #[test]
    fn works_getlasterror0() {
        let lib = Library::new("kernel32.dll").unwrap();
        let gle: Symbol<unsafe extern "system" fn() -> DWORD> = unsafe {
            lib.get(b"GetLastError\0").unwrap()
        };
        unsafe {
            errhandlingapi::SetLastError(42);
            assert_eq!(errhandlingapi::GetLastError(), gle())
        }
    }

    #[test]
    fn library_this_get() {
        use std::sync::atomic::{AtomicBool, Ordering};
        static VARIABLE: AtomicBool = AtomicBool::new();

        #[no_mangle]
        extern "C" fn library_this_get_test_fn() {
            VARIABLE.store(true, Ordering::SeqCst);
        }
        let lib = Library::this().expect("this library");
        let test_fn: Symbol<unsafe extern "C" fn()> = unsafe {
            lib.get(b"library_this_get_test_fn\0").expect("get symbol");
        };
        assert_eq!(VARIABLE.load(Ordering::SeqCst), false);
        test_fn();
        assert_eq!(VARIABLE.load(Ordering::SeqCst), true);
    }
}
