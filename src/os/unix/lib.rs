use error::LibraryClose;
use error::LibraryFindSymbol;
use error::LibraryOpen;
use os::unix::external;
use os::unix::OkOrDlerror;
use os::unix::RTLD_LAZY;
use SharedlibError as E;
use SharedlibResult as R;
use util;
use std::mem;
use std::path::Path;
use std::os::raw::c_char;
use std::os::raw::c_void;

#[derive(Debug)]
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

        util::error_guard(
            || {
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
            }
        ).ok_or_dlerror("dlopen")
        .map_err(
            |err| {
                let err = LibraryOpen::new(Box::new(err), path_to_lib.as_ref().to_path_buf());
                E::from(err)
            }
        )
    }

    pub unsafe fn find<T, TStr>(&self, symbol_str: TStr) -> R<*const T>
        where TStr: AsRef<str> {
        let symbol = symbol_str.as_ref();
        let symbol = symbol.as_ptr();
        let symbol = symbol as *const c_char;

        util::error_guard(
            || {
                let symbol = external::dlsym(self.handle, symbol);
                if symbol.is_null() {
                    None
                } else {
                    Some(mem::transmute(symbol))
                }
            }
        ).ok_or_dlerror("dlsym")
        .map_err(
            |err| {
                let err = LibraryFindSymbol::new(Box::new(err), symbol_str.as_ref().to_string());
                E::from(err)
            }
        )
    }
}

impl Drop for Lib {
    fn drop(&mut self) {
        util::error_guard(
            || {
                if unsafe { external::dlclose(self.handle) } == 0 {
                    Some(())
                } else {
                    None
                }
            }
        ).ok_or_dlerror("dlclose")
        .map_err(
            |err| {
                let err = LibraryClose::new(Box::new(err));
                E::from(err)
            }
        ).unwrap();
    }
}
