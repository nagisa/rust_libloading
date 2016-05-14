use std::error::Error;
use std::ffi::NulError;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::io::Error as IoError;
use std::io::ErrorKind as IoErrorKind;

#[derive(Debug)]
pub struct NullError {
    null_position: usize
}

impl NullError {
    pub fn new(null_position: usize) -> Self {
        NullError { null_position: null_position }
    }
}

impl From<NulError> for NullError {
    fn from(e: NulError) -> NullError {
        NullError { null_position: e.nul_position() }
    }
}

impl From<NullError> for IoError {
    fn from(e: NullError) -> IoError {
        IoError::new(IoErrorKind::Other, format!("{}", e))
    }
}

impl Error for NullError {
    fn description(&self) -> &str { "non-final null byte found" }
}

impl Display for NullError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "non-final null byte at {}", self.null_position)
    }
}
