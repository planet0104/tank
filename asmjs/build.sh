cargo build --target asmjs-unknown-emscripten --release;
./delete ./target/asmjs-unknown-emscripten/release/client.js ./delete.txt;
cp ./target/asmjs-unknown-emscripten/release/client.js ../html/asmjs_client.js;
cp ./src/asmjs_imports.js ../html/asmjs_imports.js