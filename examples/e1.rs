fn main() {
    unsafe {
        libloading::Library::new("libc.so").unwrap();
    }
}
