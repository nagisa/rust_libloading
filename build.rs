#[cfg(any(target_os="linux",
          target_os="android"))]
fn main(){
    println!("cargo:rustc-link-lib=dl");
}

#[cfg(any(target_os="freebsd",
          target_os="dragonfly"))]
fn main(){
    println!("cargo:rustc-link-lib=c");
}

#[cfg(any(target_os="openbsd",
          target_os="bitrig",
          target_os="netbsd",
          target_os="macos",
          target_os="ios"))]
fn main(){
    // netbsd claims dl* will be available to any dynamically linked binary, but I haven’t found
    // any libraries that have to be linked to on other platforms.
}

#[cfg(windows)]
fn main(){
    // dependencies come with winapi.
}
