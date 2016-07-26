extern crate target_build_utils;
use target_build_utils::TargetInfo;
use std::io::Write;

fn main(){
    let target = TargetInfo::new().expect("could not get target info");
    match target.target_os() {
        "linux" | "android" => println!("cargo:rustc-link-lib=dl"),
        "freebsd" | "dragonfly" => println!("cargo:rustc-link-lib=c"),
        // netbsd claims dl* will be available to any dynamically linked binary, but I haven’t
        // found any libraries that have to be linked to on other platforms.
        // What happens if the executable is not linked up dynamically?
        "openbsd" | "bitrig" | "netbsd" | "macos" | "ios" => {}
        // dependencies come with winapi
        "windows" => {}
        tos => {
            writeln!(::std::io::stderr(),
                     "Building for an unknown target_os=`{}`!\nPlease report an issue ",
                     tos).expect("could not report the error");
            ::std::process::exit(0xfc);
        }
    }

}
