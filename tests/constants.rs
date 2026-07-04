#[cfg(all(test, unix))]
mod unix {
    const _: () = assert!(libloading::os::unix::RTLD_LOCAL == libc::RTLD_LOCAL);
    const _: () = assert!(libloading::os::unix::RTLD_GLOBAL == libc::RTLD_GLOBAL);
    const _: () = assert!(libloading::os::unix::RTLD_NOW == libc::RTLD_NOW);
    const _: () = assert!(libloading::os::unix::RTLD_LAZY == libc::RTLD_LAZY);
}
