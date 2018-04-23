cargo build --target wasm32-unknown-unknown --release
copy target\wasm32-unknown-unknown\release\client.wasm ..\html\client.wasm
::copy target\wasm32-unknown-unknown\release\client.wasm C:\nginx-1.13.12\html\client.wasm