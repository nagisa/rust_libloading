use std::ffi::{CString, NulError};
use std::os::raw;

#[derive(Debug)]
pub struct NullError(usize);

impl From<NulError> for NullError {
    fn from(e: NulError) -> NullError {
        NullError(e.nul_position())
    }
}

impl From<NullError> for ::std::io::Error {
    fn from(e: NullError) -> ::std::io::Error {
        ::std::io::Error::new(::std::io::ErrorKind::Other, format!("{}", e))
    }
}

impl ::std::error::Error for NullError {
    fn description(&self) -> &str { "non-final null byte found" }
}

impl ::std::fmt::Display for NullError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "non-final null byte at {}", self.0)
    }
}

pub enum CowCString<'a> {
    Owned(CString),
    Ref(&'a raw::c_char)
}

impl<'a> CowCString<'a> {
    /// Checks for last byte and avoids alocatting if its zero.
    ///
    /// Non-last null bytes still result in an error.
    pub fn from_bytes(slice: &'a [u8]) -> Result<CowCString<'a>, NullError> {
        Ok(if slice.len() == 0 {
            static ZERO: raw::c_char = 0;
            CowCString::Ref(&ZERO)
        } else if let Some(&0) = slice.last() {
            // check for inner nulls
            for (c, i) in slice.iter().zip(0..slice.len()-2) {
                if *c == 0 {
                    return Err(NullError(i));
                }
            }
            CowCString::Ref(unsafe {::std::mem::transmute(slice.get_unchecked(0))})
        } else {
            CowCString::Owned(try!(CString::new(slice)))
        })
    }
}

pub trait CStringAsRef {
    fn cstring_ref(&self) -> &raw::c_char;
}

impl CStringAsRef for CString {
    fn cstring_ref(&self) -> &raw::c_char {
        unsafe { &*self.as_ptr() }
    }
}

impl<'a> CStringAsRef for CowCString<'a> {
    fn cstring_ref(&self) -> &raw::c_char {
        match *self {
            CowCString::Ref(r) => r,
            CowCString::Owned(ref o) => o.cstring_ref()
        }
    }
}
