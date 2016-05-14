#[cfg(not(any(target_os="android", target_os="ios", target_os="macos")))]
pub const LIBM: &'static str = "libm.so.6";

#[cfg(target_os="android")]
pub const LIBM: &'static str = "libm.so";

#[cfg(any(target_os="ios", target_os="macos"))]
pub const LIBM: &'static str = "libm.dylib";
