use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use string::error::os_error as string;

#[derive(Debug)]
pub struct OsError {
    cause: String,
    function_called: String,
}

impl OsError {
    pub fn new(cause: String, function_called: String) -> Self {
        OsError {
            cause: cause,
            function_called: function_called,
        }
    }
}

impl Display for OsError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}{}{}",
            string::display_1(),
            self.function_called,
            string::display_2(),
            self.cause,
        )
    }
}

impl Error for OsError {
    fn description(&self) -> &str {
        string::description()
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}
