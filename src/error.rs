use alloc::ffi::CString;
use core::ffi::CStr;

/// A `dlerror` error.
pub struct DlError(pub(crate) CString);

impl core::error::Error for DlError {}

impl core::fmt::Debug for DlError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.0, f)
    }
}

impl core::fmt::Display for DlError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0.to_string_lossy())
    }
}

impl From<&CStr> for DlError {
    fn from(value: &CStr) -> Self {
        Self(value.into())
    }
}

/// A Windows API error.
#[derive(Copy, Clone)]
pub struct WindowsError(pub(crate) i32);

impl core::error::Error for WindowsError { }

impl core::fmt::Debug for WindowsError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.0, f)
    }
}

impl core::fmt::Display for WindowsError {
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let error = std::io::Error::from_raw_os_error(self.0);
        core::fmt::Display::fmt(&error, f)
    }
    #[cfg(not(feature = "std"))]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("OS error {}", self.0))
    }
}

/// Errors.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The `dlopen` call failed.
    DlOpen {
        /// The source error.
        source: DlError,
    },
    /// The `dlopen` call failed and system did not report an error.
    DlOpenUnknown,
    /// The `dlsym` call failed.
    DlSym {
        /// The source error.
        source: DlError,
    },
    /// The `dlsym` call failed and system did not report an error.
    DlSymUnknown,
    /// The `dlclose` call failed.
    DlClose {
        /// The source error.
        source: DlError,
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
    /// Input symbol of filename contains interior 0/null elements.
    InteriorZeroElements,
}

impl core::error::Error for Error {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        use Error::*;
        match self {
            LoadLibraryExW { source }
            | GetModuleHandleExW { source }
            | GetProcAddress { source }
            | FreeLibrary { source } => Some(source),
            DlOpen { source } | DlSym { source } | DlClose { source } => Some(source),
            DlOpenUnknown
            | DlSymUnknown
            | DlCloseUnknown
            | LoadLibraryExWUnknown
            | GetModuleHandleExWUnknown
            | GetProcAddressUnknown
            | FreeLibraryUnknown
            | IncompatibleSize
            | InteriorZeroElements => None,
        }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        use Error::*;
        match *self {
            DlOpen { .. } => write!(f, "dlopen failed"),
            DlOpenUnknown => write!(f, "dlopen failed, but system did not report the error"),
            DlSym { .. } => write!(f, "dlsym failed"),
            DlSymUnknown => write!(f, "dlsym failed, but system did not report the error"),
            DlClose { .. } => write!(f, "dlclose failed"),
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
            InteriorZeroElements => write!(f, "interior zero element in parameter"),
            IncompatibleSize => write!(f, "requested type cannot possibly work"),
        }
    }
}
