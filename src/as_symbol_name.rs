use crate::Error;
use alloc::ffi::CString;
use alloc::string::String;
use core::ffi::CStr;

mod private {

    pub trait AsSymbolNameSeal {
        ///
        /// This function is guaranteed to error or invoke the `FnOnce` parameter,
        /// and if called, return whatever the `FnOnce` returns.
        ///
        /// The pointer parameter to the `FnOnce` is guaranteed to point to a valid 0 terminated
        /// c-string.
        ///
        /// The data the pointer points to is guaranteed to live until the `FnOnce` returns.
        ///
        fn symbol_name<R>(
            self,
            function: impl FnOnce(*const core::ffi::c_char) -> Result<R, crate::Error>,
        ) -> Result<R, crate::Error>;
    }
}

/// This trait is implemented on all types where libloading can derive a symbol name from.
/// It is sealed and cannot be implemented by a user of libloading.
///
pub trait AsSymbolName: private::AsSymbolNameSeal {}

impl<T> AsSymbolName for T where T: private::AsSymbolNameSeal {}

impl private::AsSymbolNameSeal for &str {
    fn symbol_name<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_bytes().symbol_name(function)
    }
}

impl private::AsSymbolNameSeal for &String {
    fn symbol_name<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_str().symbol_name(function)
    }
}

impl private::AsSymbolNameSeal for String {
    fn symbol_name<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        let mut data = self.into_bytes();
        if let Some(position) = crate::util::find_interior_element(&data, 0) {
            return Err(Error::InteriorZeroElements { position });
        }

        if data.last() != Some(&0) {
            data.push(0);
        }

        function(data.as_ptr().cast())
    }
}

impl private::AsSymbolNameSeal for &CStr {
    fn symbol_name<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        function(self.as_ptr())
    }
}

impl private::AsSymbolNameSeal for &CString {
    fn symbol_name<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        function(self.as_ptr())
    }
}

impl private::AsSymbolNameSeal for CString {
    fn symbol_name<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        function(self.as_ptr())
    }
}

impl private::AsSymbolNameSeal for &[u8] {
    fn symbol_name<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        if let Some(position) = crate::util::find_interior_element(self, 0) {
            return Err(Error::InteriorZeroElements { position });
        }

        if self.last() != Some(&0) {
            let copy = crate::util::copy_and_push(self, 0);
            return function(copy.as_ptr().cast());
        }

        function(self.as_ptr().cast())
    }
}

impl<const N: usize> private::AsSymbolNameSeal for [u8; N] {
    fn symbol_name<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_slice().symbol_name(function)
    }
}

impl<const N: usize> private::AsSymbolNameSeal for &[u8; N] {
    fn symbol_name<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        self.as_slice().symbol_name(function)
    }
}

/// This implementation requires that the buffer contains valid data to call [`String::from_utf16`].
impl private::AsSymbolNameSeal for &[u16] {
    fn symbol_name<R>(
        self,
        function: impl FnOnce(*const core::ffi::c_char) -> Result<R, Error>,
    ) -> Result<R, Error> {
        let string = String::from_utf16(self)?;
        string.symbol_name(function)
    }
}
