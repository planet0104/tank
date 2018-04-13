extern crate serde_json;
extern crate tank;
use std::cell::RefCell;
use tank::{KeyEvent, TankGame, CLIENT_HEIGHT, CLIENT_WIDTH};
use tank::{RES_LG_EXPLOSION_BITMAP, RES_MISSILE_BITMAP, RES_SM_EXPLOSION__BITMAP, RES_TANK_BITMAP};
use tank::utils::Timer;
use tank::engine::CanvasContext;
use {console_log, current_time_millis, fill_rect, fill_style, fill_text, load_resource,
     request_animation_frame, set_canvas_font, set_canvas_height, set_canvas_style_height,
     set_canvas_style_margin, set_canvas_style_width, set_canvas_width, set_frame_callback,
     set_on_keydown_listener, set_on_keyup_listener, set_on_resource_load_listener,
     set_on_window_resize_listener, window_inner_height, window_inner_width};

struct Env {
    timer: Timer,
    game: TankGame,
}

struct Context2D {}

impl CanvasContext for Context2D {
    fn draw_image_at(&self, res_id: i32, x: i32, y: i32) {
        //ctx.drawImage(res_map.get(resId), x, y);
    }

    fn draw_image(
        &self,
        res_id: i32,
        source_x: i32,
        source_y: i32,
        source_width: i32,
        source_height: i32,
        dest_x: i32,
        dest_y: i32,
        dest_width: i32,
        dest_height: i32,
    ) {

    }

    fn fill_style(&self, style: &str) {}

    fn fill_rect(&self, x: i32, y: i32, width: i32, height: i32) {}

    fn fill_text(&self, text: &str, x: i32, y: i32) {}
}

thread_local!{
    static ENV: RefCell<Env> = RefCell::new(Env{
        timer:Timer::new(1, ||{ current_time_millis() }),
        game:TankGame::new(),
    });
}

pub fn start() {
    console_log("游戏启动!!!");
    set_canvas_width(CLIENT_WIDTH);
    set_canvas_height(CLIENT_HEIGHT);
    resize_window();
    set_canvas_font("32px 微软雅黑");

    set_on_window_resize_listener(|| {
        resize_window();
    });
    set_on_keydown_listener(|key| handle_key(KeyEvent::KeyDown, key));
    set_on_keyup_listener(|key| handle_key(KeyEvent::KeyUp, key));

    //加载游戏资源
    set_on_resource_load_listener(|num: i32, total: i32| {
        let percent = num as f32 / total as f32;
        let bar_width = (CLIENT_WIDTH as f32 / 1.5) as i32;
        let bar_height = bar_width / 10;
        let bar_left = CLIENT_WIDTH / 2 - bar_width / 2;
        let bar_top = CLIENT_HEIGHT / 2 - bar_height / 2;
        fill_style("rgb(200, 200, 200)");
        fill_rect(bar_left, bar_top, bar_width, bar_height);
        fill_style("rgb(120, 120, 255)");
        fill_rect(
            bar_left,
            bar_top,
            (bar_width as f32 * percent) as i32,
            bar_height,
        );
        fill_style("#ff0");
        fill_text(
            &format!("资源加载中({}/{})", num, total),
            bar_left + bar_width / 3,
            bar_top + bar_height / 2 + 10,
        );
        if num == total {
            //资源加载完成, 启动游戏循环
            request_animation_frame();
        }
    });

    load_resource(json!({
        RES_TANK_BITMAP.to_string() : "tank.png",
        RES_MISSILE_BITMAP.to_string() : "missile.png",
        RES_LG_EXPLOSION_BITMAP.to_string() : "lg_explosion.png",
        RES_SM_EXPLOSION__BITMAP.to_string() : "sm_explosion.png"
    }));

    //游戏循环
    let frame_callback = |timestamp| {
        ENV.with(|e| {
            let mut env = e.borrow_mut();
            if env.timer.ready_for_next_frame() {
                env.game.update_sprites();
                console_log(&format!("游戏循环 {}", timestamp));
            }
            request_animation_frame();
        });
    };

    //添加事件
    set_frame_callback(frame_callback);
}

//处理按键事件
fn handle_key(event: KeyEvent, key: String) {
    console_log(&format!("event={:?} key={}", event, key));
}

//窗口大小改变时，画布适应窗口
fn resize_window() {
    let (width, height) = (window_inner_width() - 5, window_inner_height() - 5);
    let (cwidth, cheight) = if width < height {
        //竖屏
        (
            width,
            (width as f32 / CLIENT_WIDTH as f32 * CLIENT_HEIGHT as f32) as i32,
        )
    } else {
        //横屏
        (
            (height as f32 / CLIENT_HEIGHT as f32 * CLIENT_WIDTH as f32) as i32,
            height,
        )
    };

    set_canvas_style_width(cwidth);
    set_canvas_style_height(cheight);
    set_canvas_style_margin((width - cwidth) / 2, (height - cheight) / 2, 0, 0);
}
