use std::ffi::{CString, NulError};
use std::os::raw;

#[derive(Debug)]
pub struct NullError(usize);

impl From<NulError> for NullError {
    fn from(e: NulError) -> NullError {
        NullError(e.nul_position())
    }
}

impl From<NullError> for String {
    fn from(e: NullError) -> String {
        format!("{}", e)
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

pub struct CheckedCStr<'a> {
    s: Option<CString>,
    p: Option<&'a raw::c_char>
}

impl<'a> CheckedCStr<'a> {
    /// Checks for last byte and avoids alocatting if its zero.
    ///
    /// Non-last null bytes still result in an error.
    pub fn from_bytes(slice: &'a [u8]) -> Result<CheckedCStr<'a>, NullError> {
        Ok(if slice.len() == 0 {
            static ZERO: raw::c_char = 0;
            CheckedCStr {
                s: None,
                p: Some(&ZERO)
            }
        } else if let Some(&0) = slice.last() {
            // check for inner nulls
            for (c, i) in slice.iter().zip(0..slice.len()-2) {
                if *c == 0 {
                    return Err(NullError(i));
                }
            }
            CheckedCStr {
                s: None,
                p: Some(unsafe { ::std::mem::transmute(slice.get_unchecked(0)) }),
            }
        } else {
            CheckedCStr {
                s: Some(try!(CString::new(slice))),
                p: None
            }
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

impl<'a> CStringAsRef for CheckedCStr<'a> {
        fn cstring_ref(&self) -> &raw::c_char {
        if let Some(ref p) = self.p {
            *p
        } else if let Some(ref s) = self.s {
            s.cstring_ref()
        } else {
            unreachable!()
        }
    }
}
