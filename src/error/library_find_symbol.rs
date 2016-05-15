use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use string::error::library_find_symbol as string;

#[derive(Debug)]
pub struct LibraryFindSymbol {
    cause: Box<Error>,
    symbol: String,
}

impl LibraryFindSymbol {
    pub fn new(cause: Box<Error>, symbol: String) -> Self {
        LibraryFindSymbol {
            cause: cause,
            symbol: symbol,
        }
    }
}

impl Display for LibraryFindSymbol {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}{}{}",
            string::display_1(),
            self.symbol,
            string::display_2(),
            self.cause,
        )
    }
}

impl Error for LibraryFindSymbol {
    fn description(&self) -> &str {
        string::description()
    }

    fn cause(&self) -> Option<&Error> {
        Some(self.cause.as_ref())
    }
}
