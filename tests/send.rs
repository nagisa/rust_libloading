extern crate libloading;

use libloading::Symbol;

#[test]
fn check_symbol_sync() {
    fn send<S: Sync>(_: Option<S>) {}
    send::<Symbol<fn ()>>(None);
}
