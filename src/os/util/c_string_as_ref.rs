use std::ffi::CString;
use std::os::raw::c_char;

pub trait CStringAsRef {
    fn cstring_ref(&self) -> &c_char;
}

impl CStringAsRef for CString {
    fn cstring_ref(&self) -> &c_char {
        unsafe { &*self.as_ptr() }
    }
}
