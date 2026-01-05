use libloading::{Library, Symbol};

const TARGET_DIR: Option<&'static str> = option_env!("CARGO_TARGET_DIR");
const TARGET_TMPDIR: Option<&'static str> = option_env!("CARGO_TARGET_TMPDIR");
const MANIFEST_DIR: &'static str = env!("CARGO_MANIFEST_DIR");

pub fn lib_path() -> std::path::PathBuf {
    [
        TARGET_TMPDIR.unwrap_or(TARGET_DIR.unwrap_or("target")),
        "libtest_helpers.module",
    ]
    .iter()
    .collect()
}

pub fn make_helpers() {
    static ONCE: ::std::sync::Once = ::std::sync::Once::new();
    ONCE.call_once(|| {
        if std::env::var_os("PRECOMPILED_TEST_HELPER").is_some() {
            //I can't be asked to make rustc work in wine.
            //I can call it myself from my linux host and then just move the file here this allows me to skip this.
            eprintln!("WILL NOT COMPILE TEST HELPERS, PROGRAM WILL ASSUME THAT {} EXISTS AND WAS EXTERNALLY PRE COMPILED", lib_path().display());
            return;
        }

        let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        let mut cmd = ::std::process::Command::new(rustc);
        cmd.arg("src/test_helpers.rs").arg("-o").arg(lib_path());
        if let Some(target) = std::env::var_os("TARGET") {
            cmd.arg("--target").arg(target);
        } else {
            eprintln!("WARNING: $TARGET NOT SPECIFIED! BUILDING HELPER MODULE FOR NATIVE TARGET.");
        }
        if cfg!(target_env = "msvc") {
            cmd.arg(format!("-Clink-arg=/DEF:{MANIFEST_DIR}/tests/ordinals.def"));
        }
        assert!(cmd
            .status()
            .expect("could not compile the test helpers!")
            .success());
    });
}

#[cfg(not(windows))]
fn is_wine() -> bool {
    false
}

#[cfg(windows)]
fn is_wine() -> bool {
    unsafe {
        //This detects wine, the linux runtime for windows programs.
        //Wine exposes the symbol wine_get_version in ntdll.dll; naturally, this symbol is absent on actual windows.
        let lib = Library::new("ntdll.dll").expect("open library");
        let wine: Result<Symbol<extern "C" fn() -> i32>, _> = lib.get("wine_get_version");
        if wine.is_ok() {
            return true;
        }
    }

    false
}

#[test]
#[cfg(feature = "std")]
fn test_id_u32() {
    make_helpers();
    unsafe {
        let lib = Library::new(lib_path()).unwrap();
        let f: Symbol<unsafe extern "C" fn(u32) -> u32> = lib.get(b"test_identity_u32\0").unwrap();
        assert_eq!(42, f(42));
    }
}

#[cfg(feature = "std")]
#[test]
fn test_as_filename_osstring() {
    as_filename_test::<std::ffi::OsString>(lib_path().into_os_string(), "potato\0beetroot".into());
}

#[cfg(feature = "std")]
#[test]
fn test_as_filename_osstr() {
    let with_nulls = std::ffi::OsStr::new("hazelnut\0peanut");
    as_filename_test::<&std::ffi::OsStr>(lib_path().as_os_str(), with_nulls);
}

#[cfg(feature = "std")]
#[test]
fn test_as_filename_pathbuf() {
    as_filename_test::<std::path::PathBuf>(lib_path(), "orange\0grape".into());
}

#[cfg(feature = "std")]
#[test]
fn test_as_filename_path() {
    as_filename_test::<&std::path::Path>(&*lib_path(), std::path::Path::new("peach\0mango"));
}

#[cfg(feature = "std")]
#[test]
fn test_as_filename_str() {
    let path = lib_path();
    if let Some(p) = path.to_str() {
        as_filename_test::<&str>(p, "kiwi\0peach\0");
    }
}

#[cfg(feature = "std")]
#[test]
fn test_as_filename_string() {
    let path = lib_path();
    if let Some(p) = path.to_str() {
        as_filename_test::<String>(p.to_string(), "apple\0banana".to_string());
    }
}

#[cfg(feature = "std")]
fn as_filename_test<T: libloading::AsFilename>(path: T, with_interior_nulls: T) {
    make_helpers();
    unsafe {
        assert!(matches!(
            Library::new(with_interior_nulls).unwrap_err(),
            libloading::Error::InteriorZeroElements,
        ));
        let lib = Library::new(path).unwrap();
        let f: Symbol<unsafe extern "C" fn(u32) -> u32> = lib.get(b"test_identity_u32\0").unwrap();
        assert_eq!(42, f(42));
    }
}

#[cfg(feature = "std")]
#[test]
fn test_as_symbol_name_string() {
    as_symbol_name_test::<String>("test_identity_u32".to_string());
    as_symbol_name_test::<String>("test_identity_u32\0".to_string());
    as_symbol_name_test_interior_nulls::<String>("test_iden\0tity_u32".to_string());
}

#[cfg(feature = "std")]
#[test]
fn test_as_symbol_name_str() {
    as_symbol_name_test::<&str>("test_identity_u32");
    as_symbol_name_test::<&str>("test_identity_u32\0");
    as_symbol_name_test_interior_nulls::<&str>("test_iden\0tity_u32\0");
}

#[cfg(feature = "std")]
#[test]
fn test_as_symbol_name_cstr() {
    as_symbol_name_test::<&std::ffi::CStr>(c"test_identity_u32");
}

#[cfg(feature = "std")]
#[test]
fn test_as_symbol_name_cstring() {
    as_symbol_name_test::<std::ffi::CString>(c"test_identity_u32".to_owned());
}

#[cfg(feature = "std")]
#[test]
fn test_as_symbol_name_bytes() {
    as_symbol_name_test::<&[u8]>(b"test_identity_u32");
    as_symbol_name_test::<&[u8]>(b"test_identity_u32\0");
    as_symbol_name_test::<&[u8; 18]>(b"test_identity_u32\0");
    as_symbol_name_test_interior_nulls::<&[u8]>(b"test_identity\0_u32");
    as_symbol_name_test_interior_nulls::<&[u8]>(b"test\0_identity_u32");
    as_symbol_name_test_interior_nulls::<&[u8; 19]>(b"test_iden\0tity_u32\0");
}

#[cfg(feature = "std")]
fn as_symbol_name_test<T: libloading::AsSymbolName>(symbol: T) {
    make_helpers();
    unsafe {
        let lib = Library::new(lib_path()).unwrap();
        let f: Symbol<unsafe extern "C" fn(u32) -> u32> = lib.get(symbol).unwrap();
        assert_eq!(42, f(42));
    }
}

#[cfg(feature = "std")]
fn as_symbol_name_test_interior_nulls<T: libloading::AsSymbolName>(symbol: T) {
    make_helpers();
    unsafe {
        let lib = Library::new(lib_path()).unwrap();
        assert!(matches!(
            lib.get::<unsafe extern "C" fn(u32) -> u32>(symbol),
            Err(libloading::Error::InteriorZeroElements),
        ));
    }
}

#[test]
#[cfg(feature = "std")]
fn test_try_into_ptr() {
    make_helpers();
    unsafe {
        let lib = Library::new(lib_path()).unwrap();
        let f: Symbol<unsafe extern "C" fn(u32) -> u32> = lib.get(b"test_identity_u32\0").unwrap();
        let ptr: *mut std::ffi::c_void = f.try_as_raw_ptr().unwrap();
        assert!(!ptr.is_null());
        let ptr_casted: extern "C" fn(u32) -> u32 = std::mem::transmute(ptr);
        assert_eq!(42, ptr_casted(42));
    }
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
struct S {
    a: u64,
    b: u32,
    c: u16,
    d: u8,
}

#[test]
#[cfg(feature = "std")]
fn test_id_struct() {
    make_helpers();
    unsafe {
        let lib = Library::new(lib_path()).unwrap();
        let f: Symbol<unsafe extern "C" fn(S) -> S> = lib.get(b"test_identity_struct\0").unwrap();
        assert_eq!(
            S {
                a: 1,
                b: 2,
                c: 3,
                d: 4
            },
            f(S {
                a: 1,
                b: 2,
                c: 3,
                d: 4
            })
        );
    }
}

#[test]
#[allow(unpredictable_function_pointer_comparisons)]
#[cfg(feature = "std")]
fn test_0_no_0() {
    make_helpers();
    unsafe {
        let lib = Library::new(lib_path()).unwrap();
        let f: Symbol<unsafe extern "C" fn(S) -> S> = lib.get(b"test_identity_struct\0").unwrap();
        let f2: Symbol<unsafe extern "C" fn(S) -> S> = lib.get(b"test_identity_struct").unwrap();

        assert_eq!(*f, *f2);
    }
}

#[test]
fn wrong_name_fails() {
    unsafe {
        Library::new("target/this_location_is_definitely_non existent:^~")
            .err()
            .unwrap();
    }
}

#[test]
#[cfg(feature = "std")]
fn missing_symbol_fails() {
    make_helpers();
    unsafe {
        let lib = Library::new(lib_path()).unwrap();
        lib.get::<*mut ()>(b"test_does_not_exist").err().unwrap();
        lib.get::<*mut ()>(b"test_does_not_exist\0").err().unwrap();
    }
}

#[test]
#[cfg(feature = "std")]
fn interior_null_fails() {
    make_helpers();
    unsafe {
        let lib = Library::new(lib_path()).unwrap();
        lib.get::<*mut ()>(b"test_does\0_not_exist").err().unwrap();
        lib.get::<*mut ()>("test_does\0_not_exist").err().unwrap();
        lib.get::<*mut ()>(b"test\0_does_not_exist\0")
            .err()
            .unwrap();
        lib.get::<*mut ()>("test_does\0_not_exist\0").err().unwrap();
    }
}

#[test]
#[cfg(feature = "std")]
fn test_incompatible_type() {
    make_helpers();
    unsafe {
        let lib = Library::new(lib_path()).unwrap();
        assert!(match lib.get::<()>(b"test_identity_u32\0") {
            Err(libloading::Error::IncompatibleSize) => true,
            _ => false,
        })
    }
}

#[test]
#[cfg(feature = "std")]
fn test_incompatible_type_named_fn() {
    make_helpers();
    unsafe fn get<'a, T>(l: &'a Library, _: T) -> Result<Symbol<'a, T>, libloading::Error> {
        l.get::<T>(b"test_identity_u32\0")
    }
    unsafe {
        let lib = Library::new(lib_path()).unwrap();
        assert!(match get(&lib, test_incompatible_type_named_fn) {
            Err(libloading::Error::IncompatibleSize) => true,
            _ => false,
        })
    }
}

#[test]
#[cfg(feature = "std")]
fn test_static_u32() {
    make_helpers();
    unsafe {
        let lib = Library::new(lib_path()).unwrap();
        let var: Symbol<*mut u32> = lib.get(b"TEST_STATIC_U32\0").unwrap();
        **var = 42;
        let help: Symbol<unsafe extern "C" fn() -> u32> =
            lib.get(b"test_get_static_u32\0").unwrap();
        assert_eq!(42, help());
    }
}

#[test]
#[cfg(feature = "std")]
fn test_static_ptr() {
    make_helpers();
    unsafe {
        let lib = Library::new(lib_path()).unwrap();
        let var: Symbol<*mut *mut ()> = lib.get(b"TEST_STATIC_PTR\0").unwrap();
        **var = *var as *mut _;
        let works: Symbol<unsafe extern "C" fn() -> bool> =
            lib.get(b"test_check_static_ptr\0").unwrap();
        assert!(works());
    }
}

#[test]
// Something about i686-pc-windows-gnu, makes dll initialisation code call abort when it is loaded
// and unloaded many times. So far it seems like an issue with mingw, not libloading, so ignoring
// the target. Especially since it is very unlikely to be fixed given the state of support its
// support.
#[cfg(not(all(target_arch = "x86", target_os = "windows", target_env = "gnu")))]
// Cygwin returns errors on `close`.
#[cfg(not(target_os = "cygwin"))]
#[cfg(feature = "std")]
fn manual_close_many_times() {
    if is_wine() {
        // The wine runtime to run windows programs under linux
        // will run out of thread local storage indices and fail this test.
        eprintln!("DETECTED WINE RUNTIME, WILL SKIP THIS TEST");
        return;
    }

    make_helpers();
    let join_handles: Vec<_> = (0..16)
        .map(|_| {
            std::thread::spawn(|| unsafe {
                for _ in 0..10000 {
                    let lib = Library::new(lib_path()).expect("open library");
                    let _: Symbol<unsafe extern "C" fn(u32) -> u32> =
                        lib.get(b"test_identity_u32").expect("get fn");
                    lib.close().expect("close is successful");
                }
            })
        })
        .collect();
    for handle in join_handles {
        handle.join().expect("thread should succeed");
    }
}

#[cfg(unix)]
#[cfg(feature = "std")]
#[test]
fn library_this_get() {
    use libloading::os::unix::Library;
    make_helpers();
    // SAFE: functions are never called
    unsafe {
        let _lib = Library::new(lib_path()).unwrap();
        let this = Library::this();
        // Library we loaded in `_lib` (should be RTLD_LOCAL).
        assert!(this
            .get::<unsafe extern "C" fn()>(b"test_identity_u32")
            .is_err());
        // Something obscure from libc...
        // Cygwin behaves like Windows so ignore it.
        #[cfg(not(target_os = "cygwin"))]
        assert!(this.get::<unsafe extern "C" fn()>(b"freopen").is_ok());
    }
}

#[cfg(windows)]
#[cfg(feature = "std")]
#[test]
fn library_this() {
    use libloading::os::windows::Library;
    make_helpers();
    unsafe {
        // SAFE: well-known library without initialisers is loaded.
        let _lib = Library::new(lib_path()).unwrap();
        let this = Library::this().expect("this library");
        // SAFE: functions are never called.
        // Library we loaded in `_lib`.
        assert!(this
            .get::<unsafe extern "C" fn()>(b"test_identity_u32")
            .is_err());
        // Something "obscure" from kernel32...
        assert!(this.get::<unsafe extern "C" fn()>(b"GetLastError").is_err());
    }
}

#[cfg(windows)]
#[test]
fn works_getlasterror() {
    use libloading::os::windows::{Library, Symbol};
    use windows_sys::Win32::Foundation::{GetLastError, SetLastError};

    unsafe {
        let lib = Library::new("kernel32.dll").unwrap();
        let gle: Symbol<unsafe extern "system" fn() -> u32> = lib.get(b"GetLastError").unwrap();
        SetLastError(42);
        assert_eq!(GetLastError(), gle())
    }
}

#[cfg(windows)]
#[test]
fn works_getlasterror0() {
    use libloading::os::windows::{Library, Symbol};
    use windows_sys::Win32::Foundation::{GetLastError, SetLastError};

    unsafe {
        let lib = Library::new("kernel32.dll").unwrap();
        let gle: Symbol<unsafe extern "system" fn() -> u32> = lib.get(b"GetLastError\0").unwrap();
        SetLastError(42);
        assert_eq!(GetLastError(), gle())
    }
}

#[cfg(windows)]
#[test]
fn works_pin_module() {
    use libloading::os::windows::Library;

    unsafe {
        let lib = Library::new("kernel32.dll").unwrap();
        lib.pin().unwrap();
    }
}

#[cfg(windows)]
#[test]
fn library_open_already_loaded() {
    use libloading::os::windows::Library;

    // Present on Windows systems and NOT used by any other tests to prevent races.
    const LIBPATH: &str = "Msftedit.dll";

    // Not loaded yet.
    assert!(match Library::open_already_loaded(LIBPATH) {
        Err(libloading::Error::GetModuleHandleExW { .. }) => true,
        _ => false,
    });

    unsafe {
        let _lib = Library::new(LIBPATH).unwrap();
        // Loaded now.
        assert!(Library::open_already_loaded(LIBPATH).is_ok());
    }
}
