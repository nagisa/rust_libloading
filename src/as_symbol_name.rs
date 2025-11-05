use crate::Error;
use alloc::ffi::CString;
use alloc::string::String;
use core::ffi::CStr;

pub(crate) trait Sealed {
    fn symbol_name<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, crate::Error>,
    ) -> Result<R, crate::Error>;
}

/// This trait is implemented for types [`Library`](crate::Library) implementations can use to look
/// up symbols.
///
/// It is currently sealed and cannot be implemented or its methods called by users of this crate.
#[expect(private_bounds)]
pub trait AsSymbolName: Sealed {}

impl AsSymbolName for &str {}
impl Sealed for &str {
    fn symbol_name<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_bytes().symbol_name(function)
    }
}

impl AsSymbolName for &String {}
impl Sealed for &String {
    fn symbol_name<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_str().symbol_name(function)
    }
}

impl AsSymbolName for String {}
impl Sealed for String {
    fn symbol_name<R>(
        mut self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        if crate::util::check_null_bytes(self.as_bytes())? {
            function(self.as_ptr().cast())
        } else {
            self.push('\0');
            function(self.as_ptr().cast())
        }
    }
}

impl AsSymbolName for &CStr {}
impl Sealed for &CStr {
    fn symbol_name<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        function(self.as_ptr())
    }
}

impl AsSymbolName for &CString {}
impl Sealed for &CString {
    fn symbol_name<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        function(self.as_ptr())
    }
}

impl AsSymbolName for CString {}
impl Sealed for CString {
    fn symbol_name<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        function(self.as_ptr())
    }
}

impl AsSymbolName for &[u8] {}
impl Sealed for &[u8] {
    fn symbol_name<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        if crate::util::check_null_bytes(self)? {
            function(self.as_ptr().cast())
        } else {
            let copy = crate::util::copy_and_push(self, 0);
            function(copy.as_ptr().cast())
        }
    }
}

impl<const N: usize> AsSymbolName for [u8; N] {}
impl<const N: usize> Sealed for [u8; N] {
    fn symbol_name<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_slice().symbol_name(function)
    }
}

impl<const N: usize> AsSymbolName for &[u8; N] {}
impl<const N: usize> Sealed for &[u8; N] {
    fn symbol_name<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_slice().symbol_name(function)
    }
}
