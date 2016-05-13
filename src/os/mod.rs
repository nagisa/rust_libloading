#[cfg(unix)]
pub mod unix;

#[cfg(windows)]
pub mod windows;

#[cfg(unix)]
pub use os::unix::*;

#[cfg(windows)]
pub use os::windows::*;
