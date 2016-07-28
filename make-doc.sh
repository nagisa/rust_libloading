rm -rf doc/* target/* &&
cargo doc --target=x86_64-unknown-linux-gnu &&
cargo doc --target=x86_64-pc-windows-msvc &&
cp -rf target/x86_64-unknown-linux-gnu/doc/* doc/ &&
cp -rf target/x86_64-pc-windows-msvc/doc/libloading/os/windows/* ./doc/libloading/os/windows/ &&
cp -rf target/x86_64-pc-windows-msvc/doc/{winapi,kernel32} ./doc/ &&
cd doc &&
git checkout index.html
