use std::os::raw::c_char;
use std::os::raw::c_void;

#[repr(C)]
pub struct DlInfo {
  dli_fname: *const c_char,
  dli_fbase: *mut c_void,
  dli_sname: *const c_char,
  dli_saddr: *mut c_void
}
