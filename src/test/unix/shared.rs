use Func;
use Lib;
use os::unix::Lib as UnixLib;
use Symbol;
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
    let ceil: Func<extern fn(f64) -> f64> = unsafe {
        lib.find_func("ceil").unwrap()
    };
    unsafe {
        assert_eq!(ceil.get()(0.45), 1.0);
    }
}

#[test]
fn libm_ceil0() {
    let lib = Lib::new(LIBM).unwrap();
    let ceil: Func<extern fn(f64) -> f64> = unsafe {
        lib.find_func("ceil\0").unwrap()
    };
    unsafe {
        assert_eq!(ceil.get()(0.45), 1.0);
    }
}
