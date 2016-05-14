#[cfg(unix)]
#[macro_use]
extern crate lazy_static;

#[cfg(windows)]
extern crate kernel32;

#[cfg(windows)]
extern crate winapi;

pub mod os;

pub mod lib_impl;

pub mod result;

pub mod symbol;

#[cfg(test)]
pub mod test;

pub use lib_impl::Lib;

pub use symbol::data::Data;

pub use symbol::func::Func;
