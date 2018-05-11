# 编译 wasm
cargo +nightly web build --target=wasm32-unknown-unknown --release;
cp ./target/wasm32-unknown-unknown/release/client.wasm ./html/client.wasm;
cp ./target/wasm32-unknown-unknown/release/client.js ./html/client.js;