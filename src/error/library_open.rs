use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::path::PathBuf;
use string::error::library_open as string;

#[derive(Debug)]
pub struct LibraryOpen {
    cause: Box<Error>,
    path_to_lib: PathBuf,
}

impl LibraryOpen {
    pub fn new(cause: Box<Error>, path_to_lib: PathBuf) -> Self {
        LibraryOpen {
            cause: cause,
            path_to_lib: path_to_lib,
        }
    }
}

impl Display for LibraryOpen {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}{}{}",
            string::display_1(),
            self.path_to_lib.to_string_lossy(),
            string::display_2(),
            self.cause,
        )
    }
}

impl Error for LibraryOpen {
    fn description(&self) -> &str {
        string::description()
    }

    fn cause(&self) -> Option<&Error> {
        Some(self.cause.as_ref())
    }
}
