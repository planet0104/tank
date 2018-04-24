cargo build --target wasm32-unknown-unknown --release;
cp ./target/wasm32-unknown-unknown/release/client.wasm ../html/client.wasm;
cp ./src/wasm_client.js ../html/wasm_client.js