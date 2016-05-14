use os::unix::external;
use os::unix::RTLD_LAZY;
use os::unix::RTLD_NOW;
use os::unix::util;
use os::util::CowCString;
use os::util::CStringAsRef;
use result::Result as R;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::mem;
use std::os::raw::c_int;
use std::os::raw::c_void;
use std::os::unix::ffi::OsStrExt;
use std::ptr;

/// A platform-specific equivalent of the cross-platform `Lib`.
pub struct Lib {
    handle: *mut c_void
}

impl Lib {
    fn open<P>(filename: Option<P>, flags: c_int) -> R<Lib>
        where P: AsRef<OsStr> {
        let filename = match filename {
            None => None,
            Some(ref f) => Some(try!(CowCString::from_bytes(f.as_ref().as_bytes()))),
        };
        util::with_dlerror(move || {
            let result = unsafe {
                let r = external::dlopen(match filename {
                    None => ptr::null(),
                    Some(ref f) => f.cstring_ref()
                }, flags);
                // ensure filename livess until dlopen completes
                drop(filename);
                r
            };
            if result.is_null() {
                None
            } else {
                Some(Lib {
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
    pub fn new<P: AsRef<OsStr>>(filename: P) -> R<Lib> {
        Lib::open(Some(filename), RTLD_LAZY)
    }

    /// Load the dynamic libraries linked into main program.
    ///
    /// This allows retrieving symbols from any **dynamic** library linked into the program,
    /// without specifying the exact library.
    ///
    /// Corresponds to `dlopen(NULL)`.
    pub fn this() -> Lib {
        Lib::open(None::<&OsStr>, RTLD_NOW).unwrap()
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
        // `dlsym` may return nullptr in two cases: when a symbol genuinely points to a null
        // pointer or the symbol cannot be found. In order to detect this case a double dlerror
        // pattern must be used, which is, sadly, a little bit racy.
        //
        // We try to leave as little space as possible for this to occur, but we can’t exactly
        // fully prevent it.
        let symbol_result =
            util::with_dlerror(
                || {
                    external::dlerror();
                    let symbol = external::dlsym(self.handle, symbol.cstring_ref());
                    if symbol.is_null() {
                        None
                    } else {
                        Some(symbol)
                    }
                }
            );
        match symbol_result {
            Err(None) => panic!(),
            Err(Some(e)) => Err(e),
            Ok(symbol) => Ok(mem::transmute(symbol)),
        }
    }
}

impl Drop for Lib {
    fn drop(&mut self) {
        util::with_dlerror(|| if unsafe { external::dlclose(self.handle) } == 0 {
            Some(())
        } else {
            None
        }).unwrap();
    }
}

impl Debug for Lib {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str(&format!("Lib@{:p}", self.handle))
    }
}
