use util::ERROR_MUTEX;

pub fn error_guard<TFn, T>(func: TFn) -> T
    where TFn: FnOnce() -> T {
    let _lock = ERROR_MUTEX.lock().unwrap();
    let result = func();
    drop(_lock);
    result
}
