mod get_last_error;

mod get_last_error_fail;

mod sharedlib_error;

mod sharedlib_result;

pub use error::get_last_error::GetLastError;

pub use error::get_last_error_fail::GetLastErrorFail;

pub use error::sharedlib_error::SharedlibError;

pub use error::sharedlib_result::SharedlibResult;
