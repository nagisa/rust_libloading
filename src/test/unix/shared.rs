use Lib;
use os::unix::Lib as UnixLib;
use std::mem;
use test::unix::LIBM;

#[test]
fn this() {
    UnixLib::this();
}

#[test]
fn new_libm() {
    Lib::new(LIBM).unwrap();
}

#[test]
fn new_m() {
    Lib::new("m").err().unwrap();
}

#[test]
fn libm_ceil() {
    let lib = Lib::new(LIBM).unwrap();
    let ceil: extern fn(f64) -> f64 = unsafe {
        mem::transmute(lib.get::<u8>(b"ceil").unwrap())
    };
    assert_eq!(ceil(0.45), 1.0);
}

#[test]
fn libm_ceil0() {
    let lib = Lib::new(LIBM).unwrap();
    let ceil: extern fn(f64) -> f64 = unsafe {
        mem::transmute(lib.get::<u8>(b"ceil\0").unwrap())
    };
    assert_eq!(ceil(0.45), 1.0);
}
