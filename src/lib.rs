//! A cross-platform library loader which is realistic about safety guarantees.
//! ([crates.io](https://crates.io/crates/sharedlib)) ([github](https://github.com/Tyleo/shared_lib))
//!
//! # Loading a library
//! To load a library you can use any of the [Lib](struct.Lib.html), [LibTracked](struct.LibTracked.html), or [LibUnsafe](struct.LibUnsafe.html) `structs`. Each of these `struct`s provides different guarantees. For more information about the guarantees they provide, see the [chosing your guarantees](index.html#choosing-your-guarantees) section, below. We use [Lib](struct.Lib.html) for the examples below.
//!
//! ### Calling a function in another library
//! ```
//! let path_to_lib = "examplelib.dll";
//! let lib = unsafe { try!(Lib::new(path_to_lib)) };
//! let hello_world_symbol = unsafe { try!(lib.find_func::<extern "C" fn()>(b"hello_world")) };
//! let hello_world = unsafe { hello_world_symbol.get() };
//! unsafe { hello_world(); }
//! ```
//!
//! ### Accessing data in another library
//! ```
//! let path_to_lib = "examplelib.dll";
//! let lib = unsafe { try!(Lib::new(path_to_lib)) };
//! let my_usize_symbol = unsafe { try!(lib.find_data::<usize>(b"my_usize")) };
//! let my_usize = unsafe { my_usize_symbol.get() };
//! assert_eq!(*my_usize, 0);
//! ```
//!
//! ### Choosing your guarantees
//! A common problem when loading a shared library at runtime is that a symbol may be accessed after its library has been unloaded. [sharedlib](index.html) attempts to prevent this by allowing the lifetime of the library to be tracked. Each of the different libraries, [Lib](struct.Lib.html), [LibTracked](struct.LibTracked.html), or [LibUnsafe](struct.LibUnsafe.html), provides a different tracking mechanism. Below is a small overview. For more information, see the struct level documentation.
//!
//! * [LibUnsafe](struct.LibUnsafe.html) does not provide any tracking at all. This requires no overhead but responsibility falls on the client to be sure that the library is still alive when its symbols are used.
//!
//! * [Lib](struct.Lib.html) attaches its own lifetime to each symbol it returns. This requires no overhead but it can be difficult to store the returned symbol in a `struct` because the `struct` must have a trackable lifetime which outlives the [Lib](struct.Lib.html). In other words, a struct containing a symbol must parameterize around some lifetime `a`, where `a` is less than or equal to the lifetime of the library.
//!
//! * [LibTracked](struct.LibTracked.html) returns symbols with ref-counts to the library. This requires overhead but it allows the returned symbol to be stored easily. Additionally, this `struct` is generic and can be used with `Rc`, `Arc`, or a user provided ref-count type.
//!
//! # Pitfalls
//! While [sharedlib](index.html) attempts to prevent undefined behavior, loading shared libraries is inherently unsafe. Below are some tips which you may find helpful so that your code is not exposed to undefined behavior.
//!
//! ### Avoid copying or moving symbols which contain references to memory in a loaded library
//! The [get](trait.Symbol.html#method.get) method on [Symbol](trait.Symbol.html) returns a transmuted pointer to something in a loaded library. While [sharedlib](index.html) tries to make sure that this pointer cannot outlive the library it is from, full protection is impossible. In particular: if the loaded `struct` contains pointers to things in the loaded library, and the loaded `struct` implements `Clone`, clients can clone the `struct` and make it to live longer than the library it is from. If this happens the pointers a symbol contains dangle. The example below demonstrate:
//!
//! ```
//! let some_func = {
//!     let lib = unsafe { try!(Lib::new("examplelib.dll")) };
//!     let some_func_symbol = unsafe { try!(lib.find_func::<extern "C" fn()>(b"some_func")) };
//!     // All func pointers implement `Copy` so we can duplicate one.
//!     unsafe { some_func_symbol.get() }
//!     // lib goes out of scope here.
//! };
//! // Undefined behavior
//! unsafe { some_func(); }
//! ```
//! ### Use the correct method when getting functions or data
//! Each library provides two different ways to get things from shared libraries. One way is `find_func`, and the other is `find_data`. The reason that two functions are provded is because `find_data` returns a reference to a `T` rather than a `T` itself, but for `find_func` you really just want a `T` itself. To demonstrate, suppose we only have the `find_data` method, and we want to get a function pointer with the signature `fn()`. We are inclined to call something like `lib.find_data::<fn()>(b"some_func")`. This looks innocent enough. It searches the memory of the loaded binary and finds the address of the first line of the function `some_func`. Next, the *contents* of the first line of `some_func` are treated as a function pointer rather than the *address* of the first line of `some_func`. Finally, the first line of `some_func` is returned having been incorrectly cast into a function pointer. The example below demonstrates:
//!
//! ```
//! let lib = unsafe { try!(Lib::new("examplelib.dll")) };
//! let some_func_symbol = unsafe { try!(lib.find_data::<extern "C" fn()>(b"some_func")) };
//! // some_func actually points to a function but rust thinks it points to a function pointer.
//! let some_func = some_func_symbol.get();
//! // Undefined behavior
//! unsafe { some_func(); }
//! ```
//!
//! The correct way to do this with `find_data` is as follows:
//!
//! ```
//! let lib = unsafe { try!(Lib::new("examplelib.dll")) };
//! // Get a pointer to the block of memory at "some_func", this is the function itself.
//! let some_func_symbol = unsafe { try!(lib.find_data::<u8>(b"some_func")) };
//! // The type of some_func is &u8, a reference to the first byte of `some_func`. We can convert this into a function pointer.
//! let some_func = some_func_symbol.get();
//! let some_func_ptr: extern "C" fn() = std::mem::transmute(some_func);
//! // This works now.
//! unsafe { some_func_ptr(); }
//! ```
//!
//! For convienience, the second example is provided as the `find_func` method, which does this error-prone conversion behind the scenes.
//!
//! # Frequently asked questions
//!
//! ### What is a shared library?
//! A shared library is a set of functions and variables which can be loaded after a program has been compiled. By loading a library after compilation, the library can be recompiled or changed without recompiling the main program. Shared libraries can even be loaded at runtime. Common shared library filetypes are *.dll* for windows, *.so* for unix, and *.dylib* for osx. For more information about what a shared library is, see [wikipedia](https://en.wikipedia.org/wiki/Library_(computing)#Shared_libraries).
//!
//! ### Doesn't rust already provide linking against shared libraries?
//! While rust provides linking against shared libraries, it does not provide the ability to load them at runtime. If you only want to use shared libraries that you know about before runtime, you may find not find this crate very useful. On the other hand, if you wish to load something at runtime, like a plugin, you are in the right place.

#[macro_use]
extern crate define_error;

#[cfg(unix)]
#[macro_use]
extern crate lazy_static;

#[cfg(windows)]
extern crate kernel32;

#[cfg(windows)]
extern crate winapi;

mod error;

mod os;

mod lib_impl;

mod result;

mod symbol;

#[cfg(test)]
pub mod test;

pub use lib_impl::Lib;

pub use lib_impl::LibArc;

pub use lib_impl::LibRc;

pub use lib_impl::LibTracked;

pub use lib_impl::LibUnsafe;

pub use result::Result;

pub use symbol::Data;

pub use symbol::DataArc;

pub use symbol::DataRc;

pub use symbol::DataTracked;

pub use symbol::DataUnsafe;

pub use symbol::Func;

pub use symbol::FuncArc;

pub use symbol::FuncRc;

pub use symbol::FuncTracked;

pub use symbol::FuncUnsafe;

pub use symbol::Symbol;
