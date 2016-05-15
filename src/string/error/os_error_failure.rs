pub fn description() -> &'static str {
    "A call to a native Windows function failed without specifying a reason."
}

pub fn display_1() -> &'static str {
    "A call to a native windows function failed. While calling the kernel32 function, '"
}

pub fn display_2() -> &'static str {
    "', the call failed but kernel32::GetLastError() reported success."
}
