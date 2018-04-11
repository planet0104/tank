cargo build --target wasm32-unknown-emscripten --release
copy target\wasm32-unknown-emscripten\release\client.wasm html\client.wasm
copy target\wasm32-unknown-emscripten\release\client.js html\client.js