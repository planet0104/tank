#[macro_use]
extern crate stdweb;

// cargo build --target wasm32-unknown-emscripten

fn main(){
    stdweb::initialize();
    
    js! {
        var myCanvas = document.getElementById("draw");
        var ctx = myCanvas.getContext("2d");
        ctx.fillText("你好世界", 20, 20);
        console.log("你好世界!");
    };
    println!("你好");
}