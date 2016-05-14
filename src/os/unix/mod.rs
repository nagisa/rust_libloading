pub mod dl_info;

pub mod dlerror_mutex;

pub mod external;

pub mod lib;

pub mod rtld_lazy;

pub mod rtld_now;

pub mod util;

pub use os::unix::dl_info::DlInfo;

pub use os::unix::dlerror_mutex::DLERROR_MUTEX;

pub use os::unix::lib::Lib;

pub use os::unix::rtld_lazy::RTLD_LAZY;

pub use os::unix::rtld_now::RTLD_NOW;

//////////////////////////////////////

#[test]
fn this() {
    Library::this();
}

#[cfg(all(test,
          not(any(target_os="macos",
                  target_os="ios",
                  target_os="android"))))]
const LIBM: &'static str = "libm.so.6";

#[cfg(all(test, target_os="android"))]
const LIBM: &'static str = "libm.so";

#[cfg(all(test, any(target_os="macos",
                    target_os="ios")))]
const LIBM: &'static str = "libm.dylib";

#[test]
fn new_libm() {
    Library::new(LIBM).unwrap();
}

#[test]
fn new_m() {
    Library::new("m").err().unwrap();
}

#[test]
fn libm_ceil() {
    let lib = Library::new(LIBM).unwrap();
    let ceil: Symbol<extern fn(f64) -> f64> = unsafe {
        lib.get(b"ceil").unwrap()
    };
    assert_eq!(ceil(0.45), 1.0);
}

#[test]
fn libm_ceil0() {
    let lib = Library::new(LIBM).unwrap();
    let ceil: Symbol<extern fn(f64) -> f64> = unsafe {
        lib.get(b"ceil\0").unwrap()
    };
    assert_eq!(ceil(0.45), 1.0);
}
