use test::unix::LIBM;

#[test]
fn this() {
    Library::this();
}

#[test]
fn new_libm() {
    Library::new(LIBM).unwrap();
}

#[test]
fn new_m() {
    Library::new("m").err().unwrap();
}

#[test]
fn libm_ceil() {
    let lib = Library::new(LIBM).unwrap();
    let ceil: Symbol<extern fn(f64) -> f64> = unsafe {
        lib.get(b"ceil").unwrap()
    };
    assert_eq!(ceil(0.45), 1.0);
}

#[test]
fn libm_ceil0() {
    let lib = Library::new(LIBM).unwrap();
    let ceil: Symbol<extern fn(f64) -> f64> = unsafe {
        lib.get(b"ceil\0").unwrap()
    };
    assert_eq!(ceil(0.45), 1.0);
}
