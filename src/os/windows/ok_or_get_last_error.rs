use kernel32;
use std::io::Error;

pub trait OkOrGetLastError<T> {
    fn ok_or_get_last_error(self) -> Result<T, Error>;
}

impl <T> OkOrGetLastError<T> for Option<T> {
    fn ok_or_get_last_error(self) -> Result<T, Error> {
       match self {
           Some(some) => Ok(some),
           None => {
               match unsafe { kernel32::GetLastError() } {
                   0 => panic!(),// Return a custom error here
                   error_code => Err(Error::from_raw_os_error(error_code as i32)),
               }
           },
       }
   }
}
