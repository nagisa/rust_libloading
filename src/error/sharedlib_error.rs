use error::*;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

error!(
    SharedlibError {
        OsError,
        OsErrorFailure
    }
);
