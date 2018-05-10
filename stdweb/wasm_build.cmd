
:: 编译 wasm
cargo +nightly web build --target=wasm32-unknown-unknown --release
copy target\wasm32-unknown-unknown\release\client.wasm html\client.wasm
copy target\wasm32-unknown-unknown\release\client.js html\client.js