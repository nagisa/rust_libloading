use kernel32;
use Lib;
use winapi::DWORD;

#[test]
fn works_getlasterror() {
    let lib = Lib::new("kernel32.dll").unwrap();
    let gle: extern "system" fn() -> DWORD = unsafe {
        lib.get_func_unsafe(b"GetLastError").unwrap()
    };
    unsafe {
        kernel32::SetLastError(42);
        assert_eq!(kernel32::GetLastError(), gle())
    }
}

#[test]
fn works_getlasterror0() {
    let lib = Lib::new("kernel32.dll").unwrap();
    let gle: extern "system" fn() -> DWORD = unsafe {
        lib.get_func_unsafe(b"GetLastError").unwrap()
    };
    unsafe {
        kernel32::SetLastError(42);
        assert_eq!(kernel32::GetLastError(), gle())
    }
}

#[test]
fn fails_new_kernel23() {
    Lib::new("kernel23").err().unwrap();
}
