extern crate target_build_utils;
use target_build_utils::TargetInfo;

fn main(){
    let target = TargetInfo::new().expect("could not get target info");
    match target.target_os() {
        "linux" | "android" => println!("cargo:rustc-link-lib=dl"),
        "freebsd" | "dragonfly" => println!("cargo:rustc-link-lib=c"),

        // netbsd claims dl* will be available to any dynamically linked binary, but I haven’t
        // found any libraries that have to be linked to on other platforms.
        // "openbsd" | "bitrig" | "netbsd" | "macos" | "ios" => {}
        //
        // dependencies come with winapi
        // "windows" => {}
        _ => {}
    }

}
