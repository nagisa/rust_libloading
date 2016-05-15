use error::GetLastError;
use error::GetLastErrorFail;
use kernel32;
use SharedlibError as E;
use SharedlibResult as R;
use std::io::Error as IoError;

pub trait OkOrGetLastError<T> {
    fn ok_or_get_last_error<TStr>(self, function: TStr) -> R<T>
        where TStr: AsRef<str>;
}

impl <T> OkOrGetLastError<T> for Option<T> {
    fn ok_or_get_last_error<TStr>(self, function: TStr) -> R<T>
        where TStr: AsRef<str> {
       match self {
           Some(some) => Ok(some),
           None => {
               match unsafe { kernel32::GetLastError() } {
                   0 => {
                       let err = GetLastErrorFail::new(function.as_ref().to_string());
                       Err(E::from(err))
                   },
                   error_code => {
                       let cause = Box::new(IoError::from_raw_os_error(error_code as i32));
                       let err = GetLastError::new(cause, function.as_ref().to_string());
                       Err(E::from(err))
                   },
               }
           },
       }
   }
}
