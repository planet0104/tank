cargo build --target asmjs-unknown-emscripten --release
delete target\asmjs-unknown-emscripten\release\simpleasmjs.js delete.txt
copy target\asmjs-unknown-emscripten\release\simpleasmjs.js simpleasmjs.js
