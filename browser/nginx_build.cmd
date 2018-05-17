:: 复制静态文件
echo off
copy html\index.html C:\nginx-1.13.12\html\index.html

:: 编译 wasm
cargo +nightly web build --target=wasm32-unknown-unknown --release
copy target\wasm32-unknown-unknown\release\client.wasm C:\nginx-1.13.12\html\client.wasm
copy target\wasm32-unknown-unknown\release\client.js C:\nginx-1.13.12\html\client.js
echo ================ WASM Build Complete ============
:: 复制 asmjs
cargo +nightly build --target=asmjs-unknown-emscripten --release
copy target\asmjs-unknown-emscripten\release\client.js C:\nginx-1.13.12\html\client-asm.js
echo ================ ASMJS Build Complete ===========