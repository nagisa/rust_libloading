use os::unix::external;
use os::unix::OkOrDlerror;
use os::unix::RTLD_LAZY;
use SharedlibResult as R;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::mem;
use std::os::raw::c_char;
use std::os::raw::c_void;

/// A platform-specific equivalent of the cross-platform `Lib`.
pub struct Lib {
    handle: *mut c_void
}

impl Lib {
    /// Find and load a shared library (module).
    ///
    /// Locations where library is searched for is platform specific and can’t be adjusted
    /// portably.
    ///
    /// Corresponds to `dlopen(filename)`.
    pub fn new<TStr>(filename: TStr) -> R<Lib>
        where TStr: AsRef<OsStr> {
        let filename = filename.as_ref();
        let filename = filename.to_string_lossy();
        let filename = filename.as_ptr();
        let filename = filename as *const c_char;

        {
            let result = unsafe { external::dlopen(filename, RTLD_LAZY) };

            if result.is_null() {
                None
            } else {
                let lib =
                    Lib {
                        handle: result,
                    };
                Some(lib)
            }
        }.ok_or_dlerror("dlopen")
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
    pub unsafe fn find<T, TStr>(&self, symbol: TStr) -> R<*const T>
        where TStr: AsRef<str> {
        let symbol = symbol.as_ref();
        let symbol = symbol.as_ptr();
        let symbol = symbol as *const c_char;

        // `dlsym` may return nullptr in two cases: when a symbol genuinely points to a null
        // pointer or the symbol cannot be found. In order to detect this case a double dlerror
        // pattern must be used, which is, sadly, a little bit racy.
        //
        // We try to leave as little space as possible for this to occur, but we can’t exactly
        // fully prevent it.
        let symbol = external::dlsym(self.handle, symbol);
        if symbol.is_null() {
            None
        } else {
            Some(mem::transmute(symbol))
        }.ok_or_dlerror("dlsym")
    }
}

impl Drop for Lib {
    fn drop(&mut self) {
        if unsafe { external::dlclose(self.handle) } == 0 {
            Some(())
        } else {
            None
        }.ok_or_dlerror("dlclose").unwrap();
    }
}

impl Debug for Lib {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str(&format!("Lib@{:p}", self.handle))
    }
}
