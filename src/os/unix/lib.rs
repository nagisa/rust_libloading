use os::unix::external;
use os::unix::OkOrDlerror;
use os::unix::RTLD_LAZY;
use SharedlibResult as R;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::mem;
use std::path::Path;
use std::os::raw::c_char;
use std::os::raw::c_void;

pub struct Lib {
    handle: *mut c_void
}

impl Lib {
    pub fn new<TPath>(path_to_lib: TPath) -> R<Lib>
        where TPath: AsRef<Path> {
        let path_to_lib_str =
            path_to_lib
                .as_ref()
                .to_string_lossy();
        let path_to_lib_c_str = path_to_lib_str.as_ptr() as *const c_char;

        {
            let result = unsafe { external::dlopen(path_to_lib_c_str, RTLD_LAZY) };

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

    pub unsafe fn find<T, TStr>(&self, symbol: TStr) -> R<*const T>
        where TStr: AsRef<str> {
        let symbol = symbol.as_ref();
        let symbol = symbol.as_ptr();
        let symbol = symbol as *const c_char;

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
