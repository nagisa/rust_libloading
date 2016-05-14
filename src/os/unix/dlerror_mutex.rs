use std::sync::Mutex;

lazy_static! {
    pub static ref DLERROR_MUTEX: Mutex<()> = Mutex::new(());
}
