#[cfg(unix)]
#[macro_use]
pub mod unix;

#[cfg(windows)]
pub mod windows;

pub mod util;

#[cfg(windows)]
pub mod uses {
    pub use os::windows::Lib;
}
