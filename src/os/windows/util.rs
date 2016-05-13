use kernel32;
use std::io::Error;

pub fn with_get_last_error<T, F>(closure: F) -> Result<T, Option<Error>>
where F: FnOnce() -> Option<T> {
    closure().ok_or_else(|| {
        let error = unsafe { kernel32::GetLastError() };
        if error == 0 {
            None
        } else {
            Some(Error::from_raw_os_error(error as i32))
        }
    })
}
