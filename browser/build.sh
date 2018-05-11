# 编译 asmjs
cargo build --target=asmjs-unknown-emscripten --release
cp ./target/asmjs-unknown-emscripten/release/client.js ./html/client-asm.js
# 编译 wasm
cargo +nightly web build --target=wasm32-unknown-unknown --release
cp ./target/wasm32-unknown-unknown/release/client.wasm ./html/client.wasm
cp ./target/wasm32-unknown-unknown/release/client.js ./html/client.js