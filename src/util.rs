use crate::Error;

#[inline]
pub(crate) fn ensure_compatible_types<T, E>() -> Result<(), Error> {
    if size_of::<T>() != size_of::<E>() {
        Err(Error::IncompatibleSize)
    } else {
        Ok(())
    }
}

/// This function finds the interior index of a given element in a slice.
/// It returns the index or none if it can't find it.
/// Note: Interior means anything except the last element in the slice.
/// On empty slices this function returns None.
pub(crate) fn find_interior_element<T: Eq + Copy>(data: &[T], to_find: T) -> Option<usize> {
    for (position, element) in data
        .iter()
        .take(data.len().saturating_sub(1))
        .copied()
        .enumerate()
    {
        if element == to_find {
            return Some(position);
        }
    }

    None
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
