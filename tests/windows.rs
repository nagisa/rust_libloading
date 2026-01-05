#![cfg(windows)]
extern crate libloading;
use libloading::os::windows::*;
use std::ffi::CStr;
use std::os::raw::c_void;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn load_ordinal_lib() -> Library {
    let path = super::functions::lib_path();
    super::functions::make_helpers();
    unsafe { Library::new(path.display().to_string()).expect("Windows test dll not found") }
}

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_env = "msvc"))]
#[test]
fn test_ordinal() {
    let lib = load_ordinal_lib();
    unsafe {
        let windows: Symbol<unsafe fn() -> *const i8> = lib.get_ordinal(1).expect("function");
        assert_eq!(CStr::from_ptr(windows()).to_bytes(), b"bunny");
    }
}

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_env = "msvc"))]
#[test]
fn test_try_into_ptr() {
    let lib = load_ordinal_lib();
    unsafe {
        let windows: Symbol<unsafe fn() -> *const i8> = lib.get_ordinal(1).expect("function");
        let ptr: *mut c_void = windows.as_raw_ptr();
        assert!(!ptr.is_null());
    }
}

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_env = "msvc"))]
#[test]
fn test_ordinal_missing_fails() {
    let lib = load_ordinal_lib();
    unsafe {
        // there are a few other symbols in the test DLL
        let r: Result<Symbol<unsafe fn() -> *const i8>, _> = lib.get_ordinal(8);
        r.err().unwrap();
        let r: Result<Symbol<unsafe fn() -> *const i8>, _> = lib.get_ordinal(!0);
        r.err().unwrap();
    }
}

#[test]
fn test_new_kernel23() {
    unsafe {
        Library::new("kernel23").err().unwrap();
    }
}

#[test]
fn test_new_kernel32_no_ext() {
    unsafe {
        Library::new("kernel32").unwrap();
    }
}
