pub mod error_mode_guard;

pub mod lib;

pub mod use_errormode;

pub mod util;

pub use os::windows::error_mode_guard::ErrorModeGuard;

pub use os::windows::lib::Lib;

pub use os::windows::use_errormode::USE_ERRORMODE;

//////////////////////////////////////////////////////////////////////

#[test]
fn works_getlasterror() {
    let lib = Library::new("kernel32.dll").unwrap();
    let gle: extern "system" fn() -> winapi::DWORD = unsafe {
        mem::transmute(lib.get::<u8>(b"GetLastError").unwrap())
    };
    unsafe {
        kernel32::SetLastError(42);
        assert_eq!(kernel32::GetLastError(), gle())
    }
}

#[test]
fn works_getlasterror0() {
    let lib = Library::new("kernel32.dll").unwrap();
    let gle: extern "system" fn() -> winapi::DWORD = unsafe {
        mem::transmute(lib.get::<u8>(b"GetLastError\0").unwrap())
    };
    unsafe {
        kernel32::SetLastError(42);
        assert_eq!(kernel32::GetLastError(), gle())
    }
}

#[test]
fn fails_new_kernel23() {
    Library::new("kernel23").err().unwrap();
}
