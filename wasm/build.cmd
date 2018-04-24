cargo build --target wasm32-unknown-unknown --release
copy target\wasm32-unknown-unknown\release\client.wasm ..\html\client.wasm
copy src\wasm_client.js ..\html\wasm_client.js