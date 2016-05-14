use kernel32;
use os::windows::USE_ERRORMODE;
use winapi::DWORD;
use winapi::ERROR_CALL_NOT_IMPLEMENTED;
use std::ptr;
use std::sync::atomic::Ordering;

pub struct ErrorModeGuard {
    value: DWORD,
}

impl ErrorModeGuard {
    pub fn new() -> ErrorModeGuard {
        let mut ret = ErrorModeGuard { value: 0 };

        if !USE_ERRORMODE.load(Ordering::Acquire) {
            if unsafe { kernel32::SetThreadErrorMode(1, &mut ret.value) == 0
                        && kernel32::GetLastError() == ERROR_CALL_NOT_IMPLEMENTED } {
                USE_ERRORMODE.store(true, Ordering::Release);
            } else {
                return ret;
            }
        }
        ret.value = unsafe { kernel32::SetErrorMode(1) };
        ret
    }
}

impl Drop for ErrorModeGuard {
    fn drop(&mut self) {
        unsafe {
            if !USE_ERRORMODE.load(Ordering::Relaxed) {
                kernel32::SetThreadErrorMode(self.value, ptr::null_mut());
            } else {
                kernel32::SetErrorMode(self.value);
            }
        }
    }
}
