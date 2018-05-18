# 编译 asmjs
cargo +nightly build --target=asmjs-unknown-emscripten --release;
cp ./target/asmjs-unknown-emscripten/release/client.js ./html/client-asm.js;