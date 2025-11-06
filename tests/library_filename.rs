extern crate libloading;
#[cfg(feature = "std")]
mod test {
    use libloading::library_filename;
    use std::path::Path;

    #[cfg(any(target_os = "windows", target_os = "cygwin"))]
    const EXPECTED: &str = "audioengine.dll";
    #[cfg(any(target_os = "linux", target_os = "netbsd", target_os = "openbsd", target_os = "freebsd"))]
    const EXPECTED: &str = "libaudioengine.so";
    #[cfg(target_os = "macos")]
    const EXPECTED: &str = "libaudioengine.dylib";



    #[test]
    fn test_library_filename() {
        let name = "audioengine";
        let resolved = library_filename(name);
        assert!(Path::new(&resolved).ends_with(EXPECTED));
    }
}
