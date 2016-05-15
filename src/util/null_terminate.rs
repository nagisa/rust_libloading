pub fn null_terminate<TStr>(string: TStr) -> Option<String>
    where TStr: AsRef<str> {
    let string = string.as_ref();
    match string.len() {
        0 => {
            let result = String::from("\0");
            Some(result)
        },
        len => {
            match string.as_bytes()[len - 1] {
                b'\0' => None,
                _ => {
                    let mut result = String::from(string);
                    result.push_str("\0");
                    Some(result)
                },
            }
        },
    }
}
