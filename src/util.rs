use crate::Error;

#[inline]
pub(crate) fn ensure_compatible_types<T, E>() -> Result<(), Error> {
    if ::std::mem::size_of::<T>() != ::std::mem::size_of::<E>() {
        Err(Error::IncompatibleSize)
    } else {
        Ok(())
    }
}
