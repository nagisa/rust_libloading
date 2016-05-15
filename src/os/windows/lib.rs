use kernel32;
use os::windows::OkOrGetLastError;
use SharedlibResult as R;
use std::ffi::OsString;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;
use std::path::Path;
use util;
use winapi::HMODULE;
use winapi::LPCSTR;
use winapi::WCHAR;

pub struct Lib {
    handle: HMODULE
}

impl Lib {
    pub fn new<TPath>(path_to_lib: TPath) -> R<Lib>
        where TPath: AsRef<Path> {
        let path_to_lib_vec: Vec<_> =
            path_to_lib
                .as_ref()
                .as_os_str()
                .encode_wide()
                .chain((0..1))
                .collect();
        let path_to_lib_ptr = path_to_lib_vec.as_ptr();

        util::error_guard(
            || {
                let handle = unsafe { kernel32::LoadLibraryW(path_to_lib_ptr) };
                let lib_option =
                    if handle.is_null()  {
                        None
                    } else {
                        let lib = Lib { handle: handle };
                        Some(lib)
                    };
                lib_option.ok_or_get_last_error("LoadLibraryW")
            }
        )
    }

    pub unsafe fn find<T, TStr>(&self, symbol: TStr) -> R<*const T>
        where TStr: AsRef<str> {
        let symbol = symbol.as_ref();
        let symbol = symbol.as_ptr();
        let symbol = symbol as LPCSTR;

        util::error_guard(
            || {
                let symbol = kernel32::GetProcAddress(self.handle, symbol);
                if symbol.is_null() {
                    None
                } else {
                    Some(mem::transmute(symbol))
                }.ok_or_get_last_error("GetProcAddress")
            }
        )
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
        util::error_guard(
            || {
                if unsafe { kernel32::FreeLibrary(self.handle) == 0 } {
                    None
                } else {
                    Some(())
                }.ok_or_get_last_error("FreeLibrary").unwrap()
            }
        )
    }
}
