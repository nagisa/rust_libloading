pub fn description() -> &'static str {
    "A call to a native Windows function failed without specifying a reason."
}

pub fn display_1() -> &'static str {
    "While calling the native windows function, '"
}

pub fn display_2() -> &'static str {
    "', the function call failed. GetLastError() failed to report this error."
}
