cargo build --target wasm32-unknown-unknown --release;
cp ./target/wasm32-unknown-unknown/release/client.wasm ./html/client.wasm;