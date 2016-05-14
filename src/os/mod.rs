#[cfg(unix)]
#[macro_use]
pub mod unix;

#[cfg(windows)]
pub mod windows;

pub mod util;

#[cfg(unix)]
pub mod uses {
    pub use os::unix::Lib;
}

#[cfg(windows)]
pub mod uses {
    pub use os::windows::Lib;
}
