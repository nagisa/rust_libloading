#[cfg(libloading_docs)]
use super::os::unix as imp; // the implementation used here doesn't matter particularly much...
#[cfg(all(not(libloading_docs), unix))]
use super::os::unix as imp;
#[cfg(all(not(libloading_docs), windows))]
use super::os::windows as imp;
use super::Error;
use std::ffi::OsStr;
use std::fmt;
use std::marker;
use std::ops;
use std::os::raw;

/// Flag type for passing to [`Library::new_with_flags`]
#[cfg(unix)]
pub type LoadFlags = std::os::raw::c_int;
/// Flag type for passing to [`Library::new_with_flags`]
#[cfg(windows)]
pub type LoadFlags = u32;

/// A loaded dynamic library.
#[cfg_attr(libloading_docs, doc(cfg(any(unix, windows))))]
pub struct Library(imp::Library);

impl Library {
    /// Find and load a dynamic library.
    ///
    /// The `filename` argument may be either:
    ///
    /// * A library filename;
    /// * The absolute path to the library;
    /// * A relative (to the current working directory) path to the library.
    ///
    /// # Safety
    ///
    /// When a library is loaded, initialisation routines contained within it are executed.
    /// For the purposes of safety, the execution of these routines is conceptually the same calling an
    /// unknown foreign function and may impose arbitrary requirements on the caller for the call
    /// to be sound.
    ///
    /// Additionally, the callers of this function must also ensure that execution of the
    /// termination routines contained within the library is safe as well. These routines may be
    /// executed when the library is unloaded.
    ///
    /// # Thread-safety
    ///
    /// The implementation strives to be as MT-safe as sanely possible, however on certain
    /// platforms the underlying error-handling related APIs not always MT-safe. This library
    /// shares these limitations on those platforms. In particular, on certain UNIX targets
    /// `dlerror` is not MT-safe, resulting in garbage error messages in certain MT-scenarios.
    ///
    /// Calling this function from multiple threads is not MT-safe if used in conjunction with
    /// library filenames and the library search path is modified (`SetDllDirectory` function on
    /// Windows, `{DY,}LD_LIBRARY_PATH` environment variable on UNIX).
    ///
    /// # Platform-specific behaviour
    ///
    /// When a plain library filename is supplied, the locations in which the library is searched are
    /// platform specific and cannot be adjusted in a portable manner. See the documentation for
    /// the platform specific [`os::unix::Library::new`] and [`os::windows::Library::new`] methods
    /// for further information on library lookup behaviour.
    ///
    /// If the `filename` specifies a library filename without a path and with the extension omitted,
    /// the `.dll` extension is implicitly added on Windows.
    ///
    /// [`os::unix::Library::new`]: crate::os::unix::Library::new
    /// [`os::windows::Library::new`]: crate::os::windows::Library::new
    ///
    /// # Tips
    ///
    /// Distributing your dynamic libraries under a filename common to all platforms (e.g.
    /// `awesome.module`) allows you to avoid code which has to account for platform’s conventional
    /// library filenames.
    ///
    /// Strive to specify an absolute or at least a relative path to your library, unless
    /// system-wide libraries are being loaded. Platform-dependent library search locations
    /// combined with various quirks related to path-less filenames may cause flakiness in
    /// programs.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ::libloading::Library;
    /// // Any of the following are valid.
    /// unsafe {
    ///     let _ = Library::new("/path/to/awesome.module").unwrap();
    ///     let _ = Library::new("../awesome.module").unwrap();
    ///     let _ = Library::new("libsomelib.so.1").unwrap();
    /// }
    /// ```
    pub unsafe fn new<P: AsRef<OsStr>>(filename: P) -> Result<Library, Error> {
        imp::Library::new(filename).map(From::from)
    }

    /// Find and load a dynamic library, and specify open flags.
    ///
    /// This provides the same functionality as [`new`] and allows using the same safer types, but
    /// provides a way to control which flags are used to open the shared library. Note that flags
    /// are different on different operating systems and in order for this to be used in a
    /// cross-platform way, different flags must be specified for Unix and Windows. See constants
    /// in [`os::unix`] and [`os::windows`] for options.
    ///
    /// See [`new`] for further documentation about safe usage and platform-specific behavior.
    ///
    /// # Safety
    ///
    /// All safety requirements specified by [`new`] must be followed for `new_with_flags`.
    ///
    /// Some flags may change the ways that library initialization run, or that the libraries may
    /// be used. This must be taken into account.
    ///
    /// # Tips
    ///
    /// Unix always requires one of either [RTLD_LAZY](crate::os::unix::RTLD_LAZY) or
    /// [RTLD_NOW](crate::os::unix::RTLD_NOW) to be specified.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use libloading::{Library, os};
    ///
    /// // When debugging, relocate immediately to catch any missing symbols. Otherwise
    /// // load lazily to speed up loading
    /// #[cfg(unix)]
    /// let flags = if cfg!(debug_assertions) { os::unix::RTLD_NOW } else { os::unix::RTLD_LAZY };
    ///
    /// // Windows has no similar controls
    /// #[cfg(windows)]
    /// let flags = 0;
    /// 
    /// // SAFETY: `awesome` has no unsound initialization or exit routines
    /// unsafe {
    ///     let lib = Library::new_with_flags("/path/to/awesome.module", flags).unwrap();
    ///     // ...
    ///     # let _ = lib;
    /// }
    /// ```
    ///
    /// [`new`]: Library::new
    /// [`os::unix`]: crate::os::unix
    /// [`os::windows`]: crate::os::windows
    pub unsafe fn new_with_flags<P: AsRef<OsStr>>(
        filename: P,
        flags: LoadFlags,
    ) -> Result<Library, Error> {
        #[cfg(unix)]
        let res = imp::Library::open(Some(filename), flags);

        #[cfg(windows)]
        let res = imp::Library::load_with_flags(filename, flags);

        res.map(From::from)
    }

    /// Get a pointer to a function or static variable by symbol name.
    ///
    /// The `symbol` may not contain any null bytes, with the exception of the last byte. Providing a
    /// null-terminated `symbol` may help to avoid an allocation.
    ///
    /// The symbol is interpreted as-is; no mangling is done. This means that symbols like `x::y` are
    /// most likely invalid.
    ///
    /// # Safety
    ///
    /// Users of this API must specify the correct type of the function or variable loaded.
    ///
    /// # Platform-specific behaviour
    ///
    /// The implementation of thread-local variables is extremely platform specific and uses of such
    /// variables that work on e.g. Linux may have unintended behaviour on other targets.
    ///
    /// On POSIX implementations where the `dlerror` function is not confirmed to be MT-safe (such
    /// as FreeBSD), this function will unconditionally return an error when the underlying `dlsym`
    /// call returns a null pointer. There are rare situations where `dlsym` returns a genuine null
    /// pointer without it being an error. If loading a null pointer is something you care about,
    /// consider using the [`os::unix::Library::get_singlethreaded`] call.
    ///
    /// [`os::unix::Library::get_singlethreaded`]: crate::os::unix::Library::get_singlethreaded
    ///
    /// # Examples
    ///
    /// Given a loaded library:
    ///
    /// ```no_run
    /// # use ::libloading::Library;
    /// let lib = unsafe {
    ///     Library::new("/path/to/awesome.module").unwrap()
    /// };
    /// ```
    ///
    /// Loading and using a function looks like this:
    ///
    /// ```no_run
    /// # use ::libloading::{Library, Symbol};
    /// # let lib = unsafe {
    /// #     Library::new("/path/to/awesome.module").unwrap()
    /// # };
    /// unsafe {
    ///     let awesome_function: Symbol<unsafe extern fn(f64) -> f64> =
    ///         lib.get(b"awesome_function\0").unwrap();
    ///     awesome_function(0.42);
    /// }
    /// ```
    ///
    /// A static variable may also be loaded and inspected:
    ///
    /// ```no_run
    /// # use ::libloading::{Library, Symbol};
    /// # let lib = unsafe { Library::new("/path/to/awesome.module").unwrap() };
    /// unsafe {
    ///     let awesome_variable: Symbol<*mut f64> = lib.get(b"awesome_variable\0").unwrap();
    ///     **awesome_variable = 42.0;
    /// };
    /// ```
    pub unsafe fn get<'lib, T>(&'lib self, symbol: &[u8]) -> Result<Symbol<'lib, T>, Error> {
        self.0.get(symbol).map(|from| Symbol::from_raw(from, self))
    }

    /// Unload the library.
    ///
    /// This method might be a no-op, depending on the flags with which the `Library` was opened,
    /// what library was opened or other platform specifics.
    ///
    /// You only need to call this if you are interested in handling any errors that may arise when
    /// library is unloaded. Otherwise the implementation of `Drop` for `Library` will close the
    /// library and ignore the errors were they arise.
    ///
    /// The underlying data structures may still get leaked if an error does occur.
    pub fn close(self) -> Result<(), Error> {
        self.0.close()
    }
}

impl fmt::Debug for Library {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<imp::Library> for Library {
    fn from(lib: imp::Library) -> Library {
        Library(lib)
    }
}

impl From<Library> for imp::Library {
    fn from(lib: Library) -> imp::Library {
        lib.0
    }
}

unsafe impl Send for Library {}
unsafe impl Sync for Library {}

/// Symbol from a library.
///
/// This type is a safeguard against using dynamically loaded symbols after a `Library` is
/// unloaded. The primary method to create an instance of a `Symbol` is via [`Library::get`].
///
/// The `Deref` trait implementation allows the use of `Symbol` as if it was a function or variable
/// itself, without taking care to “extract” the function or variable manually most of the time.
///
/// [`Library::get`]: Library::get
#[cfg_attr(libloading_docs, doc(cfg(any(unix, windows))))]
pub struct Symbol<'lib, T: 'lib> {
    inner: imp::Symbol<T>,
    pd: marker::PhantomData<&'lib T>,
}

impl<'lib, T> Symbol<'lib, T> {
    /// Extract the wrapped `os::platform::Symbol`.
    ///
    /// # Safety
    ///
    /// Using this function relinquishes all the lifetime guarantees. It is up to the developer to
    /// ensure the resulting `Symbol` is not used past the lifetime of the `Library` this symbol
    /// was loaded from.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ::libloading::{Library, Symbol};
    /// unsafe {
    ///     let lib = Library::new("/path/to/awesome.module").unwrap();
    ///     let symbol: Symbol<*mut u32> = lib.get(b"symbol\0").unwrap();
    ///     let symbol = symbol.into_raw();
    /// }
    /// ```
    pub unsafe fn into_raw(self) -> imp::Symbol<T> {
        self.inner
    }

    /// Wrap the `os::platform::Symbol` into this safe wrapper.
    ///
    /// Note that, in order to create association between the symbol and the library this symbol
    /// came from, this function requires a reference to the library.
    ///
    /// # Safety
    ///
    /// The `library` reference must be exactly the library `sym` was loaded from.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ::libloading::{Library, Symbol};
    /// unsafe {
    ///     let lib = Library::new("/path/to/awesome.module").unwrap();
    ///     let symbol: Symbol<*mut u32> = lib.get(b"symbol\0").unwrap();
    ///     let symbol = symbol.into_raw();
    ///     let symbol = Symbol::from_raw(symbol, &lib);
    /// }
    /// ```
    pub unsafe fn from_raw<L>(sym: imp::Symbol<T>, library: &'lib L) -> Symbol<'lib, T> {
        let _ = library; // ignore here for documentation purposes.
        Symbol {
            inner: sym,
            pd: marker::PhantomData,
        }
    }

    /// Try to convert the symbol into a raw pointer.
    /// Success depends on the platform. Currently, this fn always succeeds and returns some.
    ///
    /// # Safety
    ///
    /// Using this function relinquishes all the lifetime guarantees. It is up to the developer to
    /// ensure the resulting `Symbol` is not used past the lifetime of the `Library` this symbol
    /// was loaded from.
    pub unsafe fn try_as_raw_ptr(self) -> Option<*mut raw::c_void> {
        Some(
            #[allow(unused_unsafe)] // 1.56.0 compat
            unsafe {
                // SAFE: the calling function has the same soundness invariants as this callee.
                self.into_raw()
            }
            .as_raw_ptr(),
        )
    }
}

impl<'lib, T> Symbol<'lib, Option<T>> {
    /// Lift Option out of the symbol.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ::libloading::{Library, Symbol};
    /// unsafe {
    ///     let lib = Library::new("/path/to/awesome.module").unwrap();
    ///     let symbol: Symbol<Option<*mut u32>> = lib.get(b"symbol\0").unwrap();
    ///     let symbol: Symbol<*mut u32> = symbol.lift_option().expect("static is not null");
    /// }
    /// ```
    pub fn lift_option(self) -> Option<Symbol<'lib, T>> {
        self.inner.lift_option().map(|is| Symbol {
            inner: is,
            pd: marker::PhantomData,
        })
    }
}

impl<'lib, T> Clone for Symbol<'lib, T> {
    fn clone(&self) -> Symbol<'lib, T> {
        Symbol {
            inner: self.inner.clone(),
            pd: marker::PhantomData,
        }
    }
}

// FIXME: implement FnOnce for callable stuff instead.
impl<'lib, T> ops::Deref for Symbol<'lib, T> {
    type Target = T;
    fn deref(&self) -> &T {
        ops::Deref::deref(&self.inner)
    }
}

impl<'lib, T> fmt::Debug for Symbol<'lib, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

unsafe impl<'lib, T: Send> Send for Symbol<'lib, T> {}
unsafe impl<'lib, T: Sync> Sync for Symbol<'lib, T> {}
