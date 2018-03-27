//参考: https://www.hellorust.com/setup/wasm-target/
mod matrix;
mod utils;
mod vector2d;

//--------------------------------------------
//-------------游戏资源ID----------------------
//--------------------------------------------
pub const RES_SPLASH_BITMAP:i32 = 0;
pub const RES_DESERT_BITMAP:i32 = 1;

//-----------------------------------
//-------------事件ID----------------
pub const EVENT_MOUSE_MOVE:i32 = 0;
pub const EVENT_MOUSE_CLICK:i32 = 1;
pub const EVENT_TOUCH_MOVE:i32 = 10;

//导入的JS帮助函数
extern {
    pub fn log(text: *const u8, len:usize);
    pub fn current_time_millis()->u64;
    pub fn random()->f64;
    pub fn add_resource(res_id:i32, url: *const u8, len:usize);
    pub fn load_resource();
    pub fn request_animation_frame();
    pub fn window_width()->i32;
    pub fn window_height()->i32;
    pub fn set_canvas_size(width:i32, height:i32);
    pub fn set_canvas_margin(left:i32, top:i32, right:i32, bottom:i32);
    pub fn set_canvas_style_size(width:i32, height:i32);
    pub fn set_canvas_font(font: *const u8, len:i32);
    pub fn canvas_offset_left()->i32;
    pub fn fill_style_rgb(r:u8, g:u8, b:u8);
    pub fn fill_rect(x:i32, y:i32, width:i32, height:i32);
    pub fn fill_text(text: *const u8, len:usize, x:i32, y:i32);
    pub fn draw_image_at(res_id:i32, x:i32, y:i32);
    pub fn draw_image(res_id:i32, source_x:i32, source_y:i32, source_width:i32, source_height:i32, dest_x:i32, dest_y:i32, dest_width:i32, dest_height:i32);
}

//----------------------------------------------
//--------------以下为导出到JS的函数-------------
//----------------------------------------------

#[no_mangle]
pub fn run() {
    let msg = "游戏开始...";
    unsafe {
        log(msg.as_ptr(), msg.len());
    }
}
#[no_mangle]
pub fn on_window_resize() {}
#[no_mangle]
pub fn on_load_resource_progress(current:i32, total:i32){}

//游戏循环主函数(由window.requestAnimationFrame调用)
#[no_mangle]
pub fn draw_frame() {
    unsafe { request_animation_frame(); }
}

#[no_mangle]
pub fn on_resources_load() {
    //资源加载完成启动游戏
}

#[no_mangle]
pub fn on_touch_event(event:i32, x:i32, y:i32){
    //处理鼠标、触摸事件
}