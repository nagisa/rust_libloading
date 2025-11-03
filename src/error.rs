use alloc::ffi::CString;
use alloc::string::FromUtf16Error;
use core::char::DecodeUtf16Error;
use core::ffi::CStr;
use core::str::Utf8Error;

/// A `dlerror` error.
pub struct DlDescription(pub(crate) CString);

impl core::fmt::Debug for DlDescription {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.0, f)
    }
}

impl From<&CStr> for DlDescription {
    fn from(value: &CStr) -> Self {
        Self(value.into())
    }
}

/// A Windows API error.
#[derive(Copy, Clone)]
pub struct WindowsError(pub(crate) i32);

impl core::fmt::Debug for WindowsError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.0, f)
    }
}

#[cfg(feature = "std")]
impl From<WindowsError> for std::io::Error {
    fn from(value: WindowsError) -> Self {
        std::io::Error::from_raw_os_error(value.0)
    }
}

/// Errors.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The `dlopen` call failed.
    DlOpen {
        /// The source error.
        desc: DlDescription,
    },
    /// The `dlopen` call failed and system did not report an error.
    DlOpenUnknown,
    /// The `dlsym` call failed.
    DlSym {
        /// The source error.
        desc: DlDescription,
    },
    /// The `dlsym` call failed and system did not report an error.
    DlSymUnknown,
    /// The `dlclose` call failed.
    DlClose {
        /// The source error.
        desc: DlDescription,
    },
    /// The `dlclose` call failed and system did not report an error.
    DlCloseUnknown,
    /// The `LoadLibraryW` call failed.
    LoadLibraryExW {
        /// The source error.
        source: WindowsError,
    },
    /// The `LoadLibraryW` call failed and system did not report an error.
    LoadLibraryExWUnknown,
    /// The `GetModuleHandleExW` call failed.
    GetModuleHandleExW {
        /// The source error.
        source: WindowsError,
    },
    /// The `GetModuleHandleExW` call failed and system did not report an error.
    GetModuleHandleExWUnknown,
    /// The `GetProcAddress` call failed.
    GetProcAddress {
        /// The source error.
        source: WindowsError,
    },
    /// The `GetProcAddressUnknown` call failed and system did not report an error.
    GetProcAddressUnknown,
    /// The `FreeLibrary` call failed.
    FreeLibrary {
        /// The source error.
        source: WindowsError,
    },
    /// The `FreeLibrary` call failed and system did not report an error.
    FreeLibraryUnknown,
    /// The requested type cannot possibly work.
    IncompatibleSize,
    /// Could not create a new CString.
    CreateCString {
        /// The source error.
        source: alloc::ffi::NulError,
    },
    /// Could not parse some sequence of bytes as utf-8.
    Utf8Error {
        /// The source error.
        source: Utf8Error,
    },
    /// Could not parse some sequence of bytes as utf-16.
    DecodeUtf16Error {
        ///The source error.
        source: DecodeUtf16Error,
    },
    /// Could not parse some sequence of bytes as utf-16.
    FromUtf16Error {
        ///The source error.
        source: FromUtf16Error,
    },
    /// Could not create a new CString from bytes with trailing null.
    CreateCStringWithTrailing {
        /// The source error.
        source: core::ffi::FromBytesWithNulError,
    },
}

impl From<Utf8Error> for Error {
    fn from(value: Utf8Error) -> Self {
        Self::Utf8Error { source: value }
    }
}

impl From<DecodeUtf16Error> for Error {
    fn from(value: DecodeUtf16Error) -> Self {
        Self::DecodeUtf16Error { source: value }
    }
}

impl From<FromUtf16Error> for Error {
    fn from(value: FromUtf16Error) -> Self {
        Self::FromUtf16Error { source: value }
    }
}

impl core::error::Error for Error {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        use Error::*;
        match *self {
            CreateCString { ref source } => Some(source),
            CreateCStringWithTrailing { ref source } => Some(source),
            _ => None,
        }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        use Error::*;
        match *self {
            DlOpen { ref desc } => write!(f, "{}", desc.0.to_string_lossy()),
            DlOpenUnknown => write!(f, "dlopen failed, but system did not report the error"),
            DlSym { ref desc } => write!(f, "{}", desc.0.to_string_lossy()),
            DlSymUnknown => write!(f, "dlsym failed, but system did not report the error"),
            DlClose { ref desc } => write!(f, "{}", desc.0.to_string_lossy()),
            DlCloseUnknown => write!(f, "dlclose failed, but system did not report the error"),
            LoadLibraryExW { .. } => write!(f, "LoadLibraryExW failed"),
            LoadLibraryExWUnknown => write!(
                f,
                "LoadLibraryExW failed, but system did not report the error"
            ),
            GetModuleHandleExW { .. } => write!(f, "GetModuleHandleExW failed"),
            GetModuleHandleExWUnknown => write!(
                f,
                "GetModuleHandleExWUnknown failed, but system did not report the error"
            ),
            GetProcAddress { .. } => write!(f, "GetProcAddress failed"),
            GetProcAddressUnknown => write!(
                f,
                "GetProcAddress failed, but system did not report the error"
            ),
            FreeLibrary { .. } => write!(f, "FreeLibrary failed"),
            FreeLibraryUnknown => {
                write!(f, "FreeLibrary failed, but system did not report the error")
            }
            CreateCString { .. } => write!(f, "could not create a C string from bytes"),

            CreateCStringWithTrailing { .. } => write!(
                f,
                "could not create a C string from bytes with trailing null"
            ),
            Utf8Error { .. } => write!(
                f,
                "could not create a C string from bytes with trailing null"
            ),
            DecodeUtf16Error { .. } => write!(f, "could not decode some bytes as utf16"),
            FromUtf16Error { .. } => write!(f, "could not parse some utf16 bytes to a string"),
            IncompatibleSize => write!(f, "requested type cannot possibly work"),
        }
    }
}
