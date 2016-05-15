/// A symbol from a shared library.
pub trait Symbol<T> {
    unsafe fn get(&self) -> T;
}
