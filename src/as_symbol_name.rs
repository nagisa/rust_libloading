use crate::{util, Error};
use alloc::ffi::CString;
use alloc::string::String;
use core::ffi::{c_char, CStr};

mod private {

    pub trait AsSymbolNameSeal {

        ///
        /// This function is guaranteed to error or invoke the `FnOnce` parameter, 
        /// and if called, return whatever the `FnOnce` returns.
        ///
        /// The pointer parameter to the `FnOnce` is guaranteed to point to a valid 0 terminated
        /// c-string.
        ///
        /// The data the pointer points to is guaranteed to live until the `FnOnce` returns.
        ///
        /// The data can be assumed to be 0 terminated utf-8 on unix or 0 terminated wtf-8 on windows.
        ///
        fn symbol_name<R>(
            &self,
            function: impl FnOnce(*const core::ffi::c_char) -> Result<R, crate::Error>,
        ) -> Result<R, crate::Error>;
    }
}

/// This trait is implemented on all types where libloading can derrive a symbol name from.
/// It is sealed and cannot be implemented by a user of libloading.
///
/// This trait is implemented for the following common types:
/// - String &String &str
/// - CString &CString &CStr
/// - OsString &OsString &OsStr
/// - PathBuf &PathBuf &Path
/// - &[u8] assumes utf8 data!
/// - &[u16] assumes utf16-ne data!
///
pub trait AsSymbolName: private::AsSymbolNameSeal {}

impl<T> AsSymbolName for T where T: private::AsSymbolNameSeal {}

impl private::AsSymbolNameSeal for &str {
    fn symbol_name<R>(
        &self,
        function: impl FnOnce(*const c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_bytes().symbol_name(function)
    }
}

impl private::AsSymbolNameSeal for &String {
    fn symbol_name<R>(
        &self,
        function: impl FnOnce(*const c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_str().symbol_name(function)
    }
}

impl private::AsSymbolNameSeal for String {
    fn symbol_name<R>(
        &self,
        function: impl FnOnce(*const c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_str().symbol_name(function)
    }
}

impl private::AsSymbolNameSeal for &CStr {
    fn symbol_name<R>(
        &self,
        function: impl FnOnce(*const c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        function(self.as_ptr())
    }
}

impl private::AsSymbolNameSeal for &CString {
    fn symbol_name<R>(
        &self,
        function: impl FnOnce(*const c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        function(self.as_ptr())
    }
}

impl private::AsSymbolNameSeal for CString {
    fn symbol_name<R>(
        &self,
        function: impl FnOnce(*const c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        function(self.as_ptr())
    }
}

impl private::AsSymbolNameSeal for &[u8] {
    fn symbol_name<R>(
        &self,
        function: impl FnOnce(*const c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        let cow = util::cstr_cow_from_bytes(self)?;
        function(cow.as_ptr())
    }
}

impl<const N: usize> private::AsSymbolNameSeal for [u8; N] {
    fn symbol_name<R>(
        &self,
        function: impl FnOnce(*const c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_slice().symbol_name(function)
    }
}

impl<const N: usize> private::AsSymbolNameSeal for &[u8; N] {
    fn symbol_name<R>(
        &self,
        function: impl FnOnce(*const c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_slice().symbol_name(function)
    }
}

impl private::AsSymbolNameSeal for &[u16] {
    fn symbol_name<R>(
        &self,
        function: impl FnOnce(*const c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        let string = String::from_utf16(self)?;
        string.symbol_name(function)
    }
}

#[cfg(feature = "std")]
#[cfg(any(windows, unix))]
mod std {
    use as_symbol_name::private;
    use std::ffi::{c_char, OsStr, OsString};
    use std::path::{Path, PathBuf};
    use Error;

    impl private::AsSymbolNameSeal for &OsStr {
        #[cfg(unix)]
        fn symbol_name<R>(
            &self,
            function: impl FnOnce(*const c_char) -> Result<R, Error>,
        ) -> Result<R, Error> {
            std::os::unix::ffi::OsStrExt::as_bytes(*self).symbol_name(function)
        }

        #[cfg(windows)]
        fn symbol_name<R>(
            &self,
            function: impl FnOnce(*const c_char) -> Result<R, Error>,
        ) -> Result<R, Error> {
            self.as_encoded_bytes().symbol_name(function)
        }
    }

    impl private::AsSymbolNameSeal for &OsString {
        fn symbol_name<R>(
            &self,
            function: impl FnOnce(*const c_char) -> Result<R, Error>,
        ) -> Result<R, Error> {
            self.as_os_str().symbol_name(function)
        }
    }

    impl private::AsSymbolNameSeal for OsString {
        fn symbol_name<R>(
            &self,
            function: impl FnOnce(*const c_char) -> Result<R, Error>,
        ) -> Result<R, Error> {
            self.as_os_str().symbol_name(function)
        }
    }

    impl private::AsSymbolNameSeal for PathBuf {
        fn symbol_name<R>(
            &self,
            function: impl FnOnce(*const c_char) -> Result<R, Error>,
        ) -> Result<R, Error> {
            self.as_os_str().symbol_name(function)
        }
    }

    impl private::AsSymbolNameSeal for &PathBuf {
        fn symbol_name<R>(
            &self,
            function: impl FnOnce(*const c_char) -> Result<R, Error>,
        ) -> Result<R, Error> {
            self.as_os_str().symbol_name(function)
        }
    }

    impl private::AsSymbolNameSeal for &Path {
        fn symbol_name<R>(
            &self,
            function: impl FnOnce(*const c_char) -> Result<R, Error>,
        ) -> Result<R, Error> {
            self.as_os_str().symbol_name(function)
        }
    }
}
