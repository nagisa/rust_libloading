#[cfg(unix)]
#[macro_use]
extern crate lazy_static;

#[cfg(windows)]
extern crate kernel32;

#[cfg(windows)]
extern crate winapi;

pub mod os;

pub mod lib_impl;

pub mod result;

pub use lib_impl::Lib;



//////////////////////////////

#[cfg(all(unix, not(any(target_os="macos", target_os="ios", target_os="android"))))]
#[test]
fn libm() {
    let lib = Library::new("libm.so.6").unwrap();
    let sin: Symbol<unsafe extern fn(f64) -> f64> = unsafe {
        lib.get(b"sin").unwrap()
    };
    assert!(unsafe { sin(::std::f64::INFINITY) }.is_nan());
    let errno: Symbol<*mut u32> = unsafe {
        lib.get(b"errno").unwrap()
    };
    assert!(unsafe { **errno } != 0);
    unsafe { **errno = 0; }
}
