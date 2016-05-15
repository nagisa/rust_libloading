use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use string::error::get_last_error_fail as string;

#[derive(Debug)]
pub struct GetLastErrorFail {
    function_called: String,
}

impl GetLastErrorFail {
    pub fn new(function_called: String) -> Self {
        GetLastErrorFail {
            function_called: function_called,
        }
    }
}

impl Display for GetLastErrorFail {
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

impl Error for GetLastErrorFail {
    fn description(&self) -> &str {
        string::description()
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}
