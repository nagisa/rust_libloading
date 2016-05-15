mod get_last_error_fail;

mod sharedlib_result;

pub use error::get_last_error_fail::GetLastErrorFail;

pub use error::sharedlib_result::SharedlibResult;

use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

error!(
    SharedlibError {
        GetLastErrorFail
    }
);
