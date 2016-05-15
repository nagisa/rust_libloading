use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use string::error::get_last_error as string;

#[derive(Debug)]
pub struct GetLastError {
    cause: Box<Error>,
    function_called: String,
}

impl GetLastError {
    pub fn new(cause: Box<Error>, function_called: String) -> Self {
        GetLastError {
            cause: cause,
            function_called: function_called,
        }
    }
}

impl Display for GetLastError {
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

impl Error for GetLastError {
    fn description(&self) -> &str {
        string::description()
    }

    fn cause(&self) -> Option<&Error> {
        Some(self.cause.as_ref())
    }
}
