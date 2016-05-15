pub mod lib;

pub mod ok_or_get_last_error;

pub use os::windows::lib::Lib;

pub use os::windows::ok_or_get_last_error::OkOrGetLastError;
