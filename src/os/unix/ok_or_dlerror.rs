use os::unix::DLERROR_MUTEX;
use os::unix::external;
use SharedlibError as E;
use SharedlibResult as R;
use std::ffi::CStr;
use std::io::Error as IoError;
use std::io::ErrorKind as IoErrorKind;

pub trait OkOrDlerror<T> {
    fn ok_or_dlerror<TStr>(self, function: TStr) -> R<T>
        where TStr: AsRef<str>;
}

impl <T> OkOrDlerror<T> for Option<T> {
    fn ok_or_dlerror<TStr>(self, function: TStr) -> R<T>
        where TStr: AsRef<str> {
        match self {
            Some(some) => Ok(some),
            None => {
                let error = unsafe { external::dlerror() };
                if error.is_null() {
                    // Can't find error.
                    panic!();
                } else {
                    let message = unsafe { CStr::from_ptr(error) };
                    // Found error.
                    panic!();
                }
            },
        }
    }
}

// First of all, whole error handling scheme in libdl is done via setting and querying some global
// state, therefore it is not safe to use libdl in MT-capable environment at all. Only in POSIX
// 2008+TC1 a thread-local state was allowed, which for our purposes is way too late.
// pub fn ok_or_dlerror<T, F>(closure: F) -> Result<T, Option<IoError>>
//     where F: FnOnce() -> Option<T> {
//     // We will guard all uses of libdl library with our own mutex. This makes libdl
//     // safe to use in MT programs provided the only way a program uses libdl is via this library.
//     let _lock = DLERROR_MUTEX.lock();
//     // While we could could call libdl here to clear the previous error value, only the dlsym
//     // depends on it being cleared beforehand and only in some cases too. We will instead clear the
//     // error inside the dlsym binding instead.
//     //
//     // In all the other cases, clearing the error here will only be hiding misuse of these bindings
//     // or the libdl.
//     closure().ok_or_else(|| unsafe {
//         // This code will only get executed if the `closure` returns `None`.
//         let error = external::dlerror();
//         if error.is_null() {
//             // In non-dlsym case this may happen when there’s bugs in our bindings or there’s
//             // non-libloading user of libdl; possibly in another thread.
//             None
//         } else {
//             // You can’t even rely on error string being static here; call to subsequent dlerror
//             // may invalidate or overwrite the error message. Why couldn’t they simply give up the
//             // ownership over the message?
//             // TODO: should do locale-aware conversion here. OTOH Rust doesn’t seem to work well in
//             // any system that uses non-utf8 locale, so I doubt there’s a problem here.
//             let message = CStr::from_ptr(error).to_string_lossy().into_owned();
//             Some(IoError::new(IoErrorKind::Other, message))
//             // Since we do a copy of the error string above, maybe we should call dlerror again to
//             // let libdl know it may free its copy of the string now?
//         }
//     })
// }
