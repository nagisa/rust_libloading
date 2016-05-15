use std::os::raw::c_char;
use std::os::raw::c_int;
use std::os::raw::c_void;

extern {
    pub fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void;

    pub fn dlclose(handle: *mut c_void) -> c_int;

    pub fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;

    pub fn dlerror() -> *mut c_char;
}
