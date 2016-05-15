use Func;
use kernel32;
use Lib;
use winapi::DWORD;
use Symbol;

#[test]
fn works_getlasterror() {
    let lib = Lib::new("kernel32.dll").unwrap();
    let gle: Func<extern "system" fn() -> DWORD> = unsafe {
        lib.find_func("GetLastError\0").unwrap()
    };
    unsafe {
        kernel32::SetLastError(42);
        assert_eq!(kernel32::GetLastError(), gle.get()())
    }
}

#[test]
fn works_getlasterror0() {
    let lib = Lib::new("kernel32.dll").unwrap();
    let gle: Func<extern "system" fn() -> DWORD> = unsafe {
        lib.find_func("GetLastError\0").unwrap()
    };
    unsafe {
        kernel32::SetLastError(42);
        assert_eq!(kernel32::GetLastError(), gle.get()())
    }
}

#[test]
fn fails_new_kernel23() {
    Lib::new("kernel23").err().unwrap();
}
