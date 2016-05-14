pub trait Symbol<T> {
    unsafe fn get(&self) -> T;
}
