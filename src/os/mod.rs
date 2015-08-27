#[cfg(any(target_os="linux",
          target_os="macos",
          target_os="freebsd",
          target_os="dragonfly",
          target_os="bitrig",
          target_os="netbsd",
          target_os="openbsd"))]
pub mod unix;
#[cfg(target_os="windows")]
pub mod windows;
