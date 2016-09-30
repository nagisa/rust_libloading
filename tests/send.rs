extern crate libloading;

use libloading::{Library, Symbol};

#[test]
fn check_library_sync() {
    fn send<S: Sync>(_: Option<S>) {}
    send::<Library>(None);
}

#[test]
fn check_symbol_sync() {
    fn send<S: Sync>(_: Option<S>) {}
    send::<Symbol<fn ()>>(None);
}
