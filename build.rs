#[cfg(target_os="linux")]
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
          target_os="macos"))]
fn main(){
    // netbsd claims dl* will be available to any dynamically linked binary, but I haven’t found
    // any libraries that have to be linked to on other platforms.
}

#[cfg(target_os="windows")]
fn main(){}
