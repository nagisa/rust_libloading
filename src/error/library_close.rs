use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use string::error::library_close as string;

#[derive(Debug)]
pub struct LibraryClose {
    cause: Box<Error>,
}

impl LibraryClose {
    pub fn new(cause: Box<Error>) -> Self {
        LibraryClose {
            cause: cause,
        }
    }
}

impl Display for LibraryClose {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}",
            string::display_1(),
            self.cause,
        )
    }
}

impl Error for LibraryClose {
    fn description(&self) -> &str {
        string::description()
    }

    fn cause(&self) -> Option<&Error> {
        Some(self.cause.as_ref())
    }
}
