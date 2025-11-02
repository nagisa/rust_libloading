use crate::Error;
use alloc::ffi::CString;
use alloc::string::String;
use core::ffi::CStr;

mod private {

    pub trait AsFilenameSeal {
        ///
        /// This function is guaranteed to error or invoke the `FnOnce` parameter,
        /// and if called, return whatever the `FnOnce` returns.
        ///
        /// The pointer parameter to the `FnOnce` is guaranteed to point to a valid "array" of
        /// undefined size which is terminated by a single '0' u16 and guaranteed to not contain
        /// interior '0' u16 elements.
        ///
        /// The data the pointer points to is guaranteed to live until the `FnOnce` returns.
        ///
        #[cfg(windows)]
        fn windows_filename<R>(
            self,
            function: impl FnOnce(*const u16) -> Result<R, crate::Error>,
        ) -> Result<R, crate::Error>;

        ///
        /// This function is guaranteed to error or invoke the `FnOnce` parameter,
        /// and if called, return whatever the `FnOnce` returns.
        ///
        /// The pointer parameter to the `FnOnce` is guaranteed to point to a valid 0 terminated
        /// c-string. The c-string is guaranteed to not contain interior '0' bytes.
        ///
        /// The data the pointer points to is guaranteed to live until the `FnOnce` returns.
        ///
        #[cfg(unix)]
        fn posix_filename<R>(
            self,
            function: impl FnOnce(*const core::ffi::c_char) -> Result<R, crate::Error>,
        ) -> Result<R, crate::Error>;
    }
}

/// This trait is implemented on all types where libloading can derive a filename from.
/// It is sealed and cannot be implemented by a user of libloading.
///
pub trait AsFilename: private::AsFilenameSeal {}

impl<T> AsFilename for T where T: private::AsFilenameSeal {}

impl private::AsFilenameSeal for &str {
    #[cfg(windows)]
    fn windows_filename<R>(
        self,
        function: impl FnOnce(*const u16) -> Result<R, Error>,
    ) -> Result<R, Error> {
        let mut utf16: alloc::vec::Vec<u16> = self.encode_utf16().collect();

        if let Some(position) = crate::util::find_interior_element(&utf16, 0) {
            return Err(Error::InteriorZeroElements { position });
        }

        if utf16.last() != Some(&0) {
            utf16.push(0);
        };

        function(utf16.as_ptr())
    }

    #[cfg(unix)]
    fn posix_filename<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_bytes().posix_filename(function)
    }
}

impl private::AsFilenameSeal for &String {
    #[cfg(windows)]
    fn windows_filename<R>(
        self,
        function: impl FnOnce(*const u16) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_str().windows_filename(function)
    }

    #[cfg(unix)]
    fn posix_filename<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_str().posix_filename(function)
    }
}

impl private::AsFilenameSeal for String {
    #[cfg(windows)]
    fn windows_filename<R>(
        self,
        function: impl FnOnce(*const u16) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_str().windows_filename(function)
    }

    #[cfg(unix)]
    fn posix_filename<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        let mut data = self.into_bytes();
        if let Some(position) = crate::util::find_interior_element(&data, 0) {
            return Err(Error::InteriorZeroElements { position });
        }

        if data.last() != Some(&0) {
            data.push(0);
        }

        function(data.as_ptr().cast())
    }
}

/// The windows implementation requires that the &CStr points to a 0 terminated utf-8 string,
/// which is valid for calling [`Cstr::to_str`].
///
/// The unix implementation has no requirements beyond those which &CStr already guarantees.
impl private::AsFilenameSeal for &CStr {
    #[cfg(windows)]
    fn windows_filename<R>(
        self,
        function: impl FnOnce(*const u16) -> Result<R, Error>,
    ) -> Result<R, Error> {
        let utf8 = self.to_str()?;
        utf8.windows_filename(function)
    }

    #[cfg(unix)]
    fn posix_filename<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        function(self.as_ptr())
    }
}

impl private::AsFilenameSeal for &CString {
    #[cfg(windows)]
    fn windows_filename<R>(
        self,
        function: impl FnOnce(*const u16) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_c_str().windows_filename(function)
    }

    #[cfg(unix)]
    fn posix_filename<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_c_str().posix_filename(function)
    }
}

impl private::AsFilenameSeal for CString {
    #[cfg(windows)]
    fn windows_filename<R>(
        self,
        function: impl FnOnce(*const u16) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_c_str().windows_filename(function)
    }

    #[cfg(unix)]
    fn posix_filename<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_c_str().posix_filename(function)
    }
}

/// For Windows the buffer must contain valid data to call [`core::str::from_utf8`].
///
/// For Unix there is no such requirement.
///
/// Both implementations further require no interior 0 bytes.
///
impl private::AsFilenameSeal for &[u8] {
    #[cfg(windows)]
    fn windows_filename<R>(
        self,
        function: impl FnOnce(*const u16) -> Result<R, Error>,
    ) -> Result<R, Error> {
        let utf8 = core::str::from_utf8(self)?;
        utf8.windows_filename(function)
    }

    #[cfg(unix)]
    fn posix_filename<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        if let Some(position) = crate::util::find_interior_element(self, 0) {
            return Err(Error::InteriorZeroElements { position });
        }

        if self.last() != Some(&0) {
            let copy = crate::util::copy_and_push(self, 0);
            return function(copy.as_ptr().cast());
        }

        function(self.as_ptr().cast())
    }
}

impl<const N: usize> private::AsFilenameSeal for [u8; N] {
    #[cfg(windows)]
    fn windows_filename<R>(
        self,
        function: impl FnOnce(*const u16) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_slice().windows_filename(function)
    }

    #[cfg(unix)]
    fn posix_filename<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_slice().posix_filename(function)
    }
}

impl<const N: usize> private::AsFilenameSeal for &[u8; N] {
    #[cfg(windows)]
    fn windows_filename<R>(
        self,
        function: impl FnOnce(*const u16) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_slice().windows_filename(function)
    }

    #[cfg(unix)]
    fn posix_filename<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_slice().posix_filename(function)
    }
}

/// For Unix the buffer must contain valid data to call [`String::from_utf16`].
///
/// For Windows there is no such requirement.
///
/// Both implementations require that the buffer contains no interior 0 elements/characters.
impl private::AsFilenameSeal for &[u16] {
    #[cfg(windows)]
    fn windows_filename<R>(
        self,
        function: impl FnOnce(*const u16) -> Result<R, Error>,
    ) -> Result<R, Error> {
        if let Some(position) = crate::util::find_interior_element(self, 0) {
            return Err(Error::InteriorZeroElements { position });
        }

        if self.last() != Some(&0) {
            let copy = crate::util::copy_and_push(self, 0);
            return function(copy.as_ptr());
        }

        function(self.as_ptr())
    }

    #[cfg(unix)]
    fn posix_filename<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        let utf8 = String::from_utf16(self)?;
        utf8.posix_filename(function)
    }
}

#[cfg(feature = "std")]
#[cfg(any(windows, unix))]
mod std {
    use crate::Error;
    use as_filename::private;
    use std::ffi::{OsStr, OsString};

    impl private::AsFilenameSeal for &OsStr {
        #[cfg(windows)]
        fn windows_filename<R>(
            self,
            function: impl FnOnce(*const u16) -> Result<R, Error>,
        ) -> Result<R, Error> {
            let mut wide: alloc::vec::Vec<u16> =
                std::os::windows::ffi::OsStrExt::encode_wide(self).collect();

            if let Some(position) = crate::util::find_interior_element(&wide, 0) {
                return Err(Error::InteriorZeroElements { position });
            }

            if wide.last() != Some(&0) {
                wide.push(0);
            };

            function(wide.as_ptr())
        }

        #[cfg(unix)]
        fn posix_filename<R>(
            self,
            function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
        ) -> Result<R, Error> {
            std::os::unix::ffi::OsStrExt::as_bytes(self).posix_filename(function)
        }
    }

    impl private::AsFilenameSeal for &OsString {
        #[cfg(windows)]
        fn windows_filename<R>(
            self,
            function: impl FnOnce(*const u16) -> Result<R, Error>,
        ) -> Result<R, Error> {
            self.as_os_str().windows_filename(function)
        }

        #[cfg(unix)]
        fn posix_filename<R>(
            self,
            function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
        ) -> Result<R, Error> {
            self.as_os_str().posix_filename(function)
        }
    }

    impl private::AsFilenameSeal for OsString {
        #[cfg(windows)]
        fn windows_filename<R>(
            self,
            function: impl FnOnce(*const u16) -> Result<R, Error>,
        ) -> Result<R, Error> {
            // This is the best we can do.
            // There is no into_wide for windows.
            // The internal repr is wtf-8 and this is different
            // from LCPWSTR that we need for the ffi calls.
            self.as_os_str().windows_filename(function)
        }

        #[cfg(unix)]
        fn posix_filename<R>(
            self,
            function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
        ) -> Result<R, Error> {
            let mut data = std::os::unix::ffi::OsStringExt::into_vec(self);
            if let Some(position) = crate::util::find_interior_element(&data, 0) {
                return Err(Error::InteriorZeroElements { position });
            }

            if data.last() != Some(&0) {
                data.push(0);
            }

            function(data.as_ptr().cast())
        }
    }

    impl private::AsFilenameSeal for std::path::PathBuf {
        #[cfg(windows)]
        fn windows_filename<R>(
            self,
            function: impl FnOnce(*const u16) -> Result<R, Error>,
        ) -> Result<R, Error> {
            self.into_os_string().windows_filename(function)
        }

        #[cfg(unix)]
        fn posix_filename<R>(
            self,
            function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
        ) -> Result<R, Error> {
            self.into_os_string().posix_filename(function)
        }
    }

    impl private::AsFilenameSeal for &std::path::PathBuf {
        #[cfg(windows)]
        fn windows_filename<R>(
            self,
            function: impl FnOnce(*const u16) -> Result<R, Error>,
        ) -> Result<R, Error> {
            self.as_os_str().windows_filename(function)
        }

        #[cfg(unix)]
        fn posix_filename<R>(
            self,
            function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
        ) -> Result<R, Error> {
            self.as_os_str().posix_filename(function)
        }
    }

    impl private::AsFilenameSeal for &std::path::Path {
        #[cfg(windows)]
        fn windows_filename<R>(
            self,
            function: impl FnOnce(*const u16) -> Result<R, Error>,
        ) -> Result<R, Error> {
            self.as_os_str().windows_filename(function)
        }

        #[cfg(unix)]
        fn posix_filename<R>(
            self,
            function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
        ) -> Result<R, Error> {
            self.as_os_str().posix_filename(function)
        }
    }
}
