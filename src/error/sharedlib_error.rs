use error::*;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

error!(
    SharedlibError {
        LibraryClose,
        LibraryFindSymbol,
        LibraryOpen,
        OsError,
        OsErrorFailure
    }
);
