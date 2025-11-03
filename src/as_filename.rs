use alloc::ffi::CString;
use alloc::string::String;
use alloc::vec::Vec;
use core::ffi::{c_char, CStr};
use util::cstr_cow_from_bytes;
use Error;

mod private {

    pub trait AsFilenameSeal {
        #[allow(unused)] //Posix doesnt use this
        fn windows_filename<R>(
            &self,
            function: impl FnOnce(*const u16) -> Result<R, crate::Error>,
        ) -> Result<R, crate::Error>;

        #[allow(unused)] //Windows doesnt use this
        fn posix_filename<R>(
            &self,
            function: impl FnOnce(*const core::ffi::c_char) -> Result<R, crate::Error>,
        ) -> Result<R, crate::Error>;
    }
}

/// This trait is implemented on all types where libloading can derrive a filename from.
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
pub trait AsFilename: private::AsFilenameSeal {}

impl<T> AsFilename for T where T: private::AsFilenameSeal {}

impl private::AsFilenameSeal for &str {
    fn windows_filename<R>(
        &self,
        function: impl FnOnce(*const u16) -> Result<R, Error>,
    ) -> Result<R, Error> {
        let mut utf16: Vec<u16> = self.encode_utf16().collect();
        if utf16.last() != Some(&0) {
            utf16.push(0);
        };
        function(utf16.as_ptr())
    }

    fn posix_filename<R>(
        &self,
        function: impl FnOnce(*const c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        let cow = cstr_cow_from_bytes(self.as_bytes())?;
        function(cow.as_ptr())
    }
}

impl private::AsFilenameSeal for &String {
    fn windows_filename<R>(
        &self,
        function: impl FnOnce(*const u16) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_str().windows_filename(function)
    }

    fn posix_filename<R>(
        &self,
        function: impl FnOnce(*const c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        let cow = cstr_cow_from_bytes(self.as_bytes())?;
        function(cow.as_ptr())
    }
}

impl private::AsFilenameSeal for String {
    fn windows_filename<R>(
        &self,
        function: impl FnOnce(*const u16) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_str().windows_filename(function)
    }

    fn posix_filename<R>(
        &self,
        function: impl FnOnce(*const c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        let cow = cstr_cow_from_bytes(self.as_bytes())?;
        function(cow.as_ptr())
    }
}

impl private::AsFilenameSeal for &CStr {
    fn windows_filename<R>(
        &self,
        function: impl FnOnce(*const u16) -> Result<R, Error>,
    ) -> Result<R, Error> {
        //We assume cstr is utf-8 here, if it's something bespoke like CESU-8 (thanks java) then yeah... no.
        let utf8 = self.to_str()?;
        utf8.windows_filename(function)
    }

    fn posix_filename<R>(
        &self,
        function: impl FnOnce(*const c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        function(self.as_ptr())
    }
}

impl private::AsFilenameSeal for &CString {
    fn windows_filename<R>(
        &self,
        function: impl FnOnce(*const u16) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_c_str().windows_filename(function)
    }

    fn posix_filename<R>(
        &self,
        function: impl FnOnce(*const c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_c_str().posix_filename(function)
    }
}

impl private::AsFilenameSeal for CString {
    fn windows_filename<R>(
        &self,
        function: impl FnOnce(*const u16) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_c_str().windows_filename(function)
    }

    fn posix_filename<R>(
        &self,
        function: impl FnOnce(*const c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_c_str().posix_filename(function)
    }
}

/// This implementation assumes that a slice always contains utf-8 bytes.
/// (which is likely the most common case if the slice originated in rust)
impl private::AsFilenameSeal for &[u8] {
    fn windows_filename<R>(
        &self,
        function: impl FnOnce(*const u16) -> Result<R, Error>,
    ) -> Result<R, Error> {
        let utf8 = core::str::from_utf8(self)?;
        utf8.windows_filename(function)
    }

    fn posix_filename<R>(
        &self,
        function: impl FnOnce(*const c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        let utf8 = core::str::from_utf8(self)?;
        utf8.posix_filename(function)
    }
}

impl<const N: usize> private::AsFilenameSeal for [u8; N] {
    fn windows_filename<R>(
        &self,
        function: impl FnOnce(*const u16) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_slice().windows_filename(function)
    }

    fn posix_filename<R>(
        &self,
        function: impl FnOnce(*const c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_slice().posix_filename(function)
    }
}

impl<const N: usize> private::AsFilenameSeal for &[u8; N] {
    fn windows_filename<R>(
        &self,
        function: impl FnOnce(*const u16) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_slice().windows_filename(function)
    }

    fn posix_filename<R>(
        &self,
        function: impl FnOnce(*const c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_slice().posix_filename(function)
    }
}

/// This implementation assumes that the slice contains utf-16 in native endian.
/// Sidenote: For windows this is always utf-16-le because the last big endian Windows system was the xbox 360 that rust doesn't support.
/// For linux this is highly likely to also be utf-16-le because big endian is only used in some old mips routers or some IBM hardware.
impl private::AsFilenameSeal for &[u16] {
    fn windows_filename<R>(
        &self,
        function: impl FnOnce(*const u16) -> Result<R, Error>,
    ) -> Result<R, Error> {
        //Check that we have valid utf-16
        for c in core::char::decode_utf16(self.iter().copied()) {
            let _ = c?;
        }

        if self.last() != Some(&0) {
            let mut copy = self.to_vec();
            copy.push(0);
            return function(copy.as_ptr());
        }

        function(self.as_ptr())
    }

    fn posix_filename<R>(
        &self,
        function: impl FnOnce(*const c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        let utf8 = String::from_utf16(self)?;
        utf8.posix_filename(function)
    }
}

#[cfg(feature = "std")]
#[cfg(any(windows, unix))]
mod std {
    use as_filename::private;
    use core::ffi::c_char;
    use std::ffi::{OsStr, OsString};
    use Error;

    impl private::AsFilenameSeal for &OsStr {
        #[cfg(unix)]
        fn windows_filename<R>(
            &self,
            _function: impl FnOnce(*const u16) -> Result<R, Error>,
        ) -> Result<R, Error> {
            panic!("windows_filename() not implemented for OsStr on posix platform");
        }

        #[cfg(windows)]
        fn windows_filename<R>(
            &self,
            function: impl FnOnce(*const u16) -> Result<R, Error>,
        ) -> Result<R, Error> {
            let mut utf16: alloc::vec::Vec<u16> =
                std::os::windows::ffi::OsStrExt::encode_wide(*self).collect();
            if utf16.last() != Some(&0) {
                utf16.push(0);
            };
            function(utf16.as_ptr())
        }

        #[cfg(unix)]
        fn posix_filename<R>(
            &self,
            function: impl FnOnce(*const c_char) -> Result<R, Error>,
        ) -> Result<R, Error> {
            let cow =
                crate::util::cstr_cow_from_bytes(std::os::unix::ffi::OsStrExt::as_bytes(*self))?;
            function(cow.as_ptr())
        }

        #[cfg(windows)]
        fn posix_filename<R>(
            &self,
            _function: impl FnOnce(*const c_char) -> Result<R, Error>,
        ) -> Result<R, Error> {
            panic!("posix_filename() not implemented for OsStr on windows")
        }
    }

    impl private::AsFilenameSeal for &OsString {
        fn windows_filename<R>(
            &self,
            function: impl FnOnce(*const u16) -> Result<R, Error>,
        ) -> Result<R, Error> {
            self.as_os_str().windows_filename(function)
        }

        fn posix_filename<R>(
            &self,
            function: impl FnOnce(*const c_char) -> Result<R, Error>,
        ) -> Result<R, Error> {
            self.as_os_str().posix_filename(function)
        }
    }

    impl private::AsFilenameSeal for OsString {
        fn windows_filename<R>(
            &self,
            function: impl FnOnce(*const u16) -> Result<R, Error>,
        ) -> Result<R, Error> {
            self.as_os_str().windows_filename(function)
        }

        fn posix_filename<R>(
            &self,
            function: impl FnOnce(*const c_char) -> Result<R, Error>,
        ) -> Result<R, Error> {
            self.as_os_str().posix_filename(function)
        }
    }

    impl private::AsFilenameSeal for std::path::PathBuf {
        fn windows_filename<R>(
            &self,
            function: impl FnOnce(*const u16) -> Result<R, Error>,
        ) -> Result<R, Error> {
            self.as_os_str().windows_filename(function)
        }

        fn posix_filename<R>(
            &self,
            function: impl FnOnce(*const c_char) -> Result<R, Error>,
        ) -> Result<R, Error> {
            self.as_os_str().posix_filename(function)
        }
    }

    impl private::AsFilenameSeal for &std::path::PathBuf {
        fn windows_filename<R>(
            &self,
            function: impl FnOnce(*const u16) -> Result<R, Error>,
        ) -> Result<R, Error> {
            self.as_os_str().windows_filename(function)
        }

        fn posix_filename<R>(
            &self,
            function: impl FnOnce(*const c_char) -> Result<R, Error>,
        ) -> Result<R, Error> {
            self.as_os_str().posix_filename(function)
        }
    }

    impl private::AsFilenameSeal for &std::path::Path {
        fn windows_filename<R>(
            &self,
            function: impl FnOnce(*const u16) -> Result<R, Error>,
        ) -> Result<R, Error> {
            self.as_os_str().windows_filename(function)
        }

        fn posix_filename<R>(
            &self,
            function: impl FnOnce(*const c_char) -> Result<R, Error>,
        ) -> Result<R, Error> {
            self.as_os_str().posix_filename(function)
        }
    }
}
