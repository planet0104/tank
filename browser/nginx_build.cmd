
:: 编译 wasm
cargo +nightly web build --target=wasm32-unknown-unknown --release
copy target\wasm32-unknown-unknown\release\client.wasm C:\nginx-1.13.12\html\client.wasm
copy target\wasm32-unknown-unknown\release\client.js C:\nginx-1.13.12\html\client.js
copy html\game.js C:\nginx-1.13.12\html\game.js