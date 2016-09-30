extern crate libloading;

use libloading::Symbol;

#[cfg(unix)]
#[test]
fn check_library_sync() {
	use libloading::Library;

    fn send<S: Sync>(_: Option<S>) {}
    send::<Library>(None);
}

#[test]
fn check_symbol_sync() {
    fn send<S: Sync>(_: Option<S>) {}
    send::<Symbol<fn ()>>(None);
}
