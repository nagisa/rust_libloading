# sharedlib

This project has been forked from rust_libloading. While rust_libloading provides a useful interface for loading shared libraries, it is inconvienient and provides incorrect safety gaurantees. This fork seeks to correct those gurantees by properly marking unsafe regions and by not requiring clients to transmute symbols.
