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

pub use lib_impl::Lib;

#[cfg(test)]
pub mod test;
