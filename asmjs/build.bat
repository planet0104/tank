cargo build --target asmjs-unknown-emscripten --release
delete target\asmjs-unknown-emscripten\release\client.js delete.txt
copy target\asmjs-unknown-emscripten\release\client.js ..\html\asmjs_client.js
copy src\asmjs_imports.js ..\html\asmjs_imports.js
