use std::sync::Mutex;

lazy_static! {
    pub static ref ERROR_MUTEX: Mutex<()> = Mutex::new(());
}
