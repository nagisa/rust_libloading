#[cfg(windows)]
extern crate winapi;

extern crate libloading;
use libloading::{Symbol, Library};

const LIBPATH: &'static str = concat!(env!("OUT_DIR"), "/libtest_helpers.module");

fn make_helpers() {
    static ONCE: ::std::sync::Once = ::std::sync::Once::new();
    ONCE.call_once(|| {
        let rustc = option_env!("RUSTC").unwrap_or_else(|| { "rustc".into() });
        let mut cmd = ::std::process::Command::new(rustc);
        cmd
            .arg("src/test_helpers.rs")
            .arg("-o")
            .arg(LIBPATH)
            .arg("--target")
            .arg(env!("LIBLOADING_TEST_TARGET"))
            .arg("-O");

        cmd
            .output()
            .expect("could not compile the test helpers!");
    });
}

#[test]
fn test_id_u32() {
    make_helpers();
    let lib = Library::new(LIBPATH).unwrap();
    unsafe {
        let f: Symbol<unsafe extern fn(u32) -> u32> = lib.get(b"test_identity_u32\0").unwrap();
        assert_eq!(42, f(42));
    }
}

#[repr(C)]
#[derive(Clone,Copy,PartialEq,Debug)]
struct S {
    a: u64,
    b: u32,
    c: u16,
    d: u8
}

#[test]
fn test_id_struct() {
    make_helpers();
    let lib = Library::new(LIBPATH).unwrap();
    unsafe {
        let f: Symbol<unsafe extern fn(S) -> S> = lib.get(b"test_identity_struct\0").unwrap();
        assert_eq!(S { a: 1, b: 2, c: 3, d: 4 }, f(S { a: 1, b: 2, c: 3, d: 4 }));
    }
}

#[test]
fn test_0_no_0() {
    make_helpers();
    let lib = Library::new(LIBPATH).unwrap();
    unsafe {
        let f: Symbol<unsafe extern fn(S) -> S> = lib.get(b"test_identity_struct\0").unwrap();
        let f2: Symbol<unsafe extern fn(S) -> S> = lib.get(b"test_identity_struct").unwrap();
        assert_eq!(*f, *f2);
    }
}

#[test]
fn wrong_name_fails() {
    Library::new(concat!(env!("OUT_DIR"), "/libtest_help")).err().unwrap();
}

#[test]
fn missing_symbol_fails() {
    make_helpers();
    let lib = Library::new(LIBPATH).unwrap();
    unsafe {
        lib.get::<*mut ()>(b"test_does_not_exist").err().unwrap();
        lib.get::<*mut ()>(b"test_does_not_exist\0").err().unwrap();
    }
}

#[test]
fn interior_null_fails() {
    make_helpers();
    let lib = Library::new(LIBPATH).unwrap();
    unsafe {
        lib.get::<*mut ()>(b"test_does\0_not_exist").err().unwrap();
        lib.get::<*mut ()>(b"test\0_does_not_exist\0").err().unwrap();
    }
}

#[test]
fn test_incompatible_type() {
    make_helpers();
    let lib = Library::new(LIBPATH).unwrap();
    unsafe {
        assert!(match lib.get::<()>(b"test_identity_u32\0") {
           Err(libloading::Error::IncompatibleSize) => true,
           _ => false,
        })
    }
}

#[test]
fn test_incompatible_type_named_fn() {
    make_helpers();
    unsafe fn get<'a, T>(l: &'a Library, _: T) -> Result<Symbol<'a, T>, libloading::Error> {
        l.get::<T>(b"test_identity_u32\0")
    }
    let lib = Library::new(LIBPATH).unwrap();
    unsafe {
        assert!(match get(&lib, test_incompatible_type_named_fn) {
           Err(libloading::Error::IncompatibleSize) => true,
           _ => false,
        })
    }
}

#[test]
fn test_static_u32() {
    make_helpers();
    let lib = Library::new(LIBPATH).unwrap();
    unsafe {
        let var: Symbol<*mut u32> = lib.get(b"TEST_STATIC_U32\0").unwrap();
        **var = 42;
        let help: Symbol<unsafe extern fn() -> u32> = lib.get(b"test_get_static_u32\0").unwrap();
        assert_eq!(42, help());
    }
}

#[test]
fn test_static_ptr() {
    make_helpers();
    let lib = Library::new(LIBPATH).unwrap();
    unsafe {
        let var: Symbol<*mut *mut ()> = lib.get(b"TEST_STATIC_PTR\0").unwrap();
        **var = *var as *mut _;
        let works: Symbol<unsafe extern fn() -> bool> =
            lib.get(b"test_check_static_ptr\0").unwrap();
        assert!(works());
    }
}

#[cfg(unix)]
#[test]
fn library_this_get() {
    use libloading::os::unix::Library;
    make_helpers();
    let _lib = Library::new(LIBPATH).unwrap();
    let this = Library::this();
    // SAFE: functions are never called
    unsafe {
        // Library we loaded in `_lib` (should be RTLD_LOCAL).
        // FIXME: inconsistent behaviour between macos and other posix systems
        // assert!(this.get::<unsafe extern "C" fn()>(b"test_identity_u32").is_err());
        // Something obscure from libc...
        assert!(this.get::<unsafe extern "C" fn()>(b"freopen").is_ok());
    }
}

#[cfg(windows)]
#[test]
fn library_this() {
    use libloading::os::windows::Library;
    make_helpers();
    let _lib = Library::new(LIBPATH).unwrap();
    let this = Library::this().expect("this library");
    // SAFE: functions are never called
    unsafe {
        // Library we loaded in `_lib`.
        assert!(this.get::<unsafe extern "C" fn()>(b"test_identity_u32").is_err());
        // Something "obscure" from kernel32...
        assert!(this.get::<unsafe extern "C" fn()>(b"GetLastError").is_err());
    }
}

#[cfg(windows)]
#[test]
fn works_getlasterror() {
    use winapi::um::errhandlingapi;
    use winapi::shared::minwindef::DWORD;
    use libloading::os::windows::{Library, Symbol};

    let lib = Library::new("kernel32.dll").unwrap();
    let gle: Symbol<unsafe extern "system" fn() -> DWORD> = unsafe {
        lib.get(b"GetLastError").unwrap()
    };
    unsafe {
        errhandlingapi::SetLastError(42);
        assert_eq!(errhandlingapi::GetLastError(), gle())
    }
}

#[cfg(windows)]
#[test]
fn works_getlasterror0() {
    use winapi::um::errhandlingapi;
    use winapi::shared::minwindef::DWORD;
    use libloading::os::windows::{Library, Symbol};

    let lib = Library::new("kernel32.dll").unwrap();
    let gle: Symbol<unsafe extern "system" fn() -> DWORD> = unsafe {
        lib.get(b"GetLastError\0").unwrap()
    };
    unsafe {
        errhandlingapi::SetLastError(42);
        assert_eq!(errhandlingapi::GetLastError(), gle())
    }
}
