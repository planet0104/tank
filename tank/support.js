//加载图片资源
function loadResources(srcMap, cb, listener){
    var resMap = new Map();
    function check(listener){
        if(listener)
            listener(resMap.size, srcMap.size);
        if(srcMap.size == resMap.size){
            cb(resMap);
        }
    }
    for(var [key, url] of srcMap.entries()){
            var image = new Image();
            image.key = key;
            image.src = url;
            image.onload = function(){
                resMap.set(this.key, this);
                check(listener);
            };
    }
}

//从wasm内存读取字符串
//offset 指针
//len 长度
function read_string(buffer, offset, len){
    const string_buffer = new Uint8Array(buffer, offset, len);
    if (typeof TextDecoder === "function"){
        return new TextDecoder("UTF-8").decode(string_buffer);
    }else{
        return decode_utf8(string_buffer);
    }
}

//是否支持webassemboy
function support_webassembly(){
    try {
        if (typeof WebAssembly === "object"
            && typeof WebAssembly.instantiate === "function") {
            const module = new WebAssembly.Module(Uint8Array.of(0x0, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00));
            if (module instanceof WebAssembly.Module)
                return new WebAssembly.Instance(module) instanceof WebAssembly.Instance;
        }
    } catch (e) {
    }
    return false;
}

//一些浏览器不支持TextDecoder,使用此方法代替
function decode_utf8(bytes) {
    //const bytes = new Uint8Array(buffer);
    let pos = 0;
    const len = bytes.length;
    const out = [];
  
    while (pos < len) {
      const byte1 = bytes[pos++];
      if (byte1 === 0) {
        break;  // NULL
      }
    
      if ((byte1 & 0x80) === 0) {  // 1-byte
        out.push(byte1);
      } else if ((byte1 & 0xe0) === 0xc0) {  // 2-byte
        const byte2 = bytes[pos++] & 0x3f;
        out.push(((byte1 & 0x1f) << 6) | byte2);
      } else if ((byte1 & 0xf0) === 0xe0) {
        const byte2 = bytes[pos++] & 0x3f;
        const byte3 = bytes[pos++] & 0x3f;
        out.push(((byte1 & 0x1f) << 12) | (byte2 << 6) | byte3);
      } else if ((byte1 & 0xf8) === 0xf0) {
        const byte2 = bytes[pos++] & 0x3f;
        const byte3 = bytes[pos++] & 0x3f;
        const byte4 = bytes[pos++] & 0x3f;
  
        // this can be > 0xffff, so possibly generate surrogates
        let codepoint = ((byte1 & 0x07) << 0x12) | (byte2 << 0x0c) | (byte3 << 0x06) | byte4;
        if (codepoint > 0xffff) {
          // codepoint &= ~0x10000;
          codepoint -= 0x10000;
          out.push((codepoint >>> 10) & 0x3ff | 0xd800)
          codepoint = 0xdc00 | codepoint & 0x3ff;
        }
        out.push(codepoint);
      } else {
        // FIXME: we're ignoring this
      }
    }
    return String.fromCharCode.apply(null, out);
  }