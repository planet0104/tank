
:: 编译 wasm
cargo +nightly web build --target=wasm32-unknown-unknown --release
copy target\wasm32-unknown-unknown\release\client.wasm C:\nginx-1.13.12\html\client.wasm
copy target\wasm32-unknown-unknown\release\client.js C:\nginx-1.13.12\html\client.js
copy html\client-asm.js C:\nginx-1.13.12\html\client-asm.js
copy html\index.html C:\nginx-1.13.12\html\index.html