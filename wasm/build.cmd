cargo  +nightly build --target wasm32-unknown-unknown --release
copy target\wasm32-unknown-unknown\release\tank.wasm ..\server\html\tank.wasm