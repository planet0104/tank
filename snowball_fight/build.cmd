:: 编译 asmjs
cargo +nightly build --target=asmjs-unknown-emscripten --release
copy target\asmjs-unknown-emscripten\release\snowball_fight.js html\snowball_fight.js