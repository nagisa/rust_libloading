use alloc::ffi::CString;
use alloc::string::FromUtf16Error;
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
    /// Could not parse some sequence of bytes as utf-8.
    Utf8Error {
        /// The source error.
        source: Utf8Error,
    },
    /// Could not parse some sequence of bytes as utf-16.
    FromUtf16Error {
        ///The source error.
        source: FromUtf16Error,
    },
    /// The data contained interior 0/null elements.
    InteriorZeroElements {
        /// The position of the interior element which was 0/null.
        position: usize,
    },
}

impl From<Utf8Error> for Error {
    fn from(value: Utf8Error) -> Self {
        Self::Utf8Error { source: value }
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
            FromUtf16Error { ref source } => Some(source),
            Utf8Error { ref source } => Some(source),
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
            Utf8Error { .. } => write!(f, "could not parse bytes as utf-8"),
            FromUtf16Error { .. } => write!(f, "could not parse some utf16 bytes to a string"),
            InteriorZeroElements { .. } => write!(f, "interior zero element in parameter"),
            IncompatibleSize => write!(f, "requested type cannot possibly work"),
        }
    }
}
