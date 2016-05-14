#[cfg(unix)]
#[macro_use]
extern crate lazy_static;

#[cfg(windows)]
extern crate kernel32;

#[cfg(windows)]
extern crate winapi;

mod os;

mod lib_impl;

mod result;

mod symbol;

#[cfg(test)]
pub mod test;

pub use lib_impl::Lib;

pub use lib_impl::LibTracked;

pub use lib_impl::LibUnsafe;

pub use result::Result;

pub use symbol::Symbol;

pub use symbol::Data;

pub use symbol::DataTracked;

pub use symbol::DataUnsafe;

pub use symbol::Func;

pub use symbol::FuncTracked;

pub use symbol::FuncUnsafe;
