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

pub use lib_impl::LibTracked;

pub use lib_impl::LibUnsafe;

pub use symbol::data::Data;

pub use symbol::data_tracked::DataTracked;

pub use symbol::data_unsafe::DataUnsafe;

pub use symbol::func::Func;

pub use symbol::func_tracked::FuncTracked;

pub use symbol::func_unsafe::FuncUnsafe;
