cargo build --target asmjs-unknown-emscripten --release
copy ./target/asmjs-unknown-emscripten/release/client.js ./html/client.js
