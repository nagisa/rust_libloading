use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use string::error::os_error_failure as string;

#[derive(Debug)]
pub struct OsErrorFailure {
    function_called: String,
}

impl OsErrorFailure {
    pub fn new(function_called: String) -> Self {
        OsErrorFailure {
            function_called: function_called,
        }
    }
}

impl Display for OsErrorFailure {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}{}",
            string::display_1(),
            self.function_called,
            string::display_2(),
        )
    }
}

impl Error for OsErrorFailure {
    fn description(&self) -> &str {
        string::description()
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}
