use os::util::CStringAsRef;
use os::util::NullError;
use std::ffi::CString;
use std::os::raw::c_char;
use std::mem;

pub enum CowCString<'a> {
    Owned(CString),
    Ref(&'a c_char)
}

impl<'a> CowCString<'a> {
    /// Checks for last byte and avoids alocatting if its zero.
    ///
    /// Non-last null bytes still result in an error.
    pub fn from_bytes(slice: &'a [u8]) -> Result<CowCString<'a>, NullError> {
        Ok(if slice.len() == 0 {
            static ZERO: c_char = 0;
            CowCString::Ref(&ZERO)
        } else if let Some(&0) = slice.last() {
            // check for inner nulls
            for (c, i) in slice.iter().zip(0..slice.len()-2) {
                if *c == 0 {
                    return Err(NullError::new(i));
                }
            }
            CowCString::Ref(unsafe {mem::transmute(slice.get_unchecked(0))})
        } else {
            CowCString::Owned(try!(CString::new(slice)))
        })
    }
}

impl<'a> CStringAsRef for CowCString<'a> {
    fn cstring_ref(&self) -> &c_char {
        match *self {
            CowCString::Ref(r) => r,
            CowCString::Owned(ref o) => o.cstring_ref()
        }
    }
}
