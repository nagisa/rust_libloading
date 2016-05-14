use std::sync::atomic::AtomicBool;
use std::sync::atomic::ATOMIC_BOOL_INIT;

pub static USE_ERRORMODE: AtomicBool = ATOMIC_BOOL_INIT;
