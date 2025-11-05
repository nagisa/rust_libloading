use crate::Error;

#[inline]
pub(crate) fn ensure_compatible_types<T, E>() -> Result<(), Error> {
    if size_of::<T>() != size_of::<E>() {
        Err(Error::IncompatibleSize)
    } else {
        Ok(())
    }
}

/// Verify that the input has no interior nulls and check whether the last byte is a null.
///
/// If any `b'\0'` at positions other than the last byte are found, an error is returned. Otherwise
/// `true` will be returned only if the last byte is `b'\0'`.
pub(crate) fn check_null_bytes(data: &[u8]) -> Result<bool, Error> {
    if let [rest @ .., last] = data {
        if rest.contains(&0) {
            Err(Error::InteriorZeroElements)
        } else {
            Ok(*last == 0)
        }
    } else {
        Ok(false)
    }
}

/// This function copies the slice into a vec and appends an element to its end.
/// The vec is allocated with reserve_exact.
pub(crate) fn copy_and_push<T: Copy>(data: &[T], to_push: T) -> alloc::vec::Vec<T> {
    let mut copy = alloc::vec::Vec::new();
    copy.reserve_exact(data.len() + 1);
    copy.extend_from_slice(data);
    copy.push(to_push);
    copy
}
