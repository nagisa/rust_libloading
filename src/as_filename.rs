use crate::Error;
use alloc::string::String;

pub(crate) trait Sealed {
    #[cfg(windows)]
    #[doc(hidden)]
    fn windows_filename<R>(
        self,
        function: impl FnOnce(*const u16) -> Result<R, crate::Error>,
    ) -> Result<R, crate::Error>;

    #[cfg(unix)]
    #[doc(hidden)]
    fn posix_filename<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, crate::Error>,
    ) -> Result<R, crate::Error>;
}

/// This trait is implemented for types that can be used as a filename when loading new
/// [`Library`](crate::Library) instances.
///
/// It is currently sealed and cannot be implemented or its methods called by users of this crate.
#[expect(private_bounds)]
pub trait AsFilename: Sealed {}

impl AsFilename for &str {}
impl Sealed for &str {
    #[cfg(windows)]
    fn windows_filename<R>(
        self,
        function: impl FnOnce(*const u16) -> Result<R, Error>,
    ) -> Result<R, Error> {
        let utf16: alloc::vec::Vec<u16> = if crate::util::check_null_bytes(self.as_bytes())? {
            self.encode_utf16().collect()
        } else {
            self.encode_utf16().chain(Some(0)).collect()
        };
        function(utf16.as_ptr())
    }

    #[cfg(unix)]
    fn posix_filename<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        if crate::util::check_null_bytes(self.as_bytes())? {
            function(self.as_ptr().cast())
        } else {
            let buffer = crate::util::copy_and_push(self.as_bytes(), 0);
            function(buffer.as_ptr().cast())
        }
    }
}

impl AsFilename for &String {}
impl Sealed for &String {
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

impl AsFilename for String {}
impl Sealed for String {
    #[cfg(windows)]
    fn windows_filename<R>(
        self,
        function: impl FnOnce(*const u16) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_str().windows_filename(function)
    }

    #[cfg(unix)]
    fn posix_filename<R>(
        mut self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        if crate::util::check_null_bytes(self.as_bytes())? {
            function(self.as_ptr().cast())
        } else {
            self.push('\0');
            function(self.as_ptr().cast())
        }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(libloading_docs, doc(cfg(feature = "std")))]
mod std {
    use super::{Sealed, AsFilename};
    use crate::Error;
    use std::ffi::{OsStr, OsString};

    impl AsFilename for &OsStr {}
    impl Sealed for &OsStr {
        #[cfg(windows)]
        fn windows_filename<R>(
            self,
            function: impl FnOnce(*const u16) -> Result<R, Error>,
        ) -> Result<R, Error> {
            use std::os::windows::ffi::OsStrExt;
            let bytes = self.as_encoded_bytes();
            let utf16: alloc::vec::Vec<u16> = if crate::util::check_null_bytes(bytes)? {
                self.encode_wide().collect()
            } else {
                self.encode_wide().chain(Some(0)).collect()
            };
            function(utf16.as_ptr())
        }

        #[cfg(unix)]
        fn posix_filename<R>(
            self,
            function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
        ) -> Result<R, Error> {
            let bytes = std::os::unix::ffi::OsStrExt::as_bytes(self);
            if crate::util::check_null_bytes(bytes)? {
                function(bytes.as_ptr().cast())
            } else {
                let buffer = crate::util::copy_and_push(bytes, 0);
                function(buffer.as_ptr().cast())
            }
        }
    }

    impl AsFilename for &OsString {}
    impl Sealed for &OsString {
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

    impl AsFilename for OsString {}
    impl Sealed for OsString {
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
            let mut bytes = std::os::unix::ffi::OsStringExt::into_vec(self);
            if crate::util::check_null_bytes(&bytes)? {
                function(bytes.as_ptr().cast())
            } else {
                bytes.push(0);
                function(bytes.as_ptr().cast())
            }
        }
    }

    impl AsFilename for std::path::PathBuf {}
    impl Sealed for std::path::PathBuf {
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

    impl AsFilename for &std::path::PathBuf {}
    impl Sealed for &std::path::PathBuf {
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

    impl AsFilename for &std::path::Path {}
    impl Sealed for &std::path::Path {
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
