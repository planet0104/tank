extern crate engine;

#[macro_use]
extern crate stdweb;

mod sprite;
use engine::background::{Background, BackgroundLayer, ScrollDir, ScrollingBackground};
use engine::canvas::Canvas;
use engine::sprite::{Rect, Sprite, BA_STOP};
use engine::{Bitmap, GameEngine, UpdateCallback};
use sprite::PersonSprite;
use std::cell::RefCell;
use std::cell::RefMut;
use std::collections::HashMap;
use std::rc::Rc;
use stdweb::web::html_element::ImageElement;

use stdweb::traits::*;
use stdweb::unstable::TryInto;
use stdweb::web::{document, window, CanvasRenderingContext2d};
use stdweb::web::event::{
    KeyPressEvent,
    KeyUpEvent,
};
use stdweb::web::html_element::CanvasElement;
use sprite::Image;

pub const WIDTH: i32 = 400;
pub const HEIGHT: i32 = 400;
const IMG_PERSON: &'static str = "character.png";

thread_local!{
    static GAME: RefCell<Option<SnowballFight>> = RefCell::new(None);
    static RESOURCES: RefCell<HashMap<String, Image>> = RefCell::new(HashMap::new());
}

pub struct SnowballFightUpdateCallback {}
impl UpdateCallback for SnowballFightUpdateCallback {
    fn on_sprite_dying(&mut self, _engine: &mut GameEngine, _idx_sprite_dying: usize) {}
    fn on_sprite_collision(
        &mut self,
        _engine: &mut GameEngine,
        _idx_sprite_hitter: usize,
        _idx_sprite_hittee: usize,
    ) -> bool {
        false
    }
}

struct SnowballFight {
    animation_callback: fn(f64),
    context2d: CanvasRenderingContext2d,
    engine: GameEngine,
    player: Rc<RefCell<PersonSprite>>,
    last_timestamp: f64,
}

impl Canvas for SnowballFight {
    fn draw_image_at(&self, bitmap: &Bitmap, x: i32, y: i32) {
        RESOURCES.with(|resources|{
            self.context2d.draw_image(resources.borrow().get(bitmap.url()).unwrap().image.clone(), x as f64, y as f64).unwrap();
        });
    }

    fn fill_style(&self, style: &str) {
        self.context2d.set_fill_style_color(style);
    }
    fn fill_rect(&self, x: i32, y: i32, width: i32, height: i32) {
        self.context2d.fill_rect(x as f64, y as f64, width as f64, height as f64);
    }

    fn translate(&self, x:f64, y:f64){
        self.context2d.translate(x, y);
    }

    fn scale(&self, x:f64, y:f64){
        self.context2d.scale(x, y);
    }

    fn save(&self){
        self.context2d.save();
    }

    fn restore(&self){
        self.context2d.restore();
    }

    fn console_log(&self, s: &str){
        console!(log, s);
    }

    fn draw_image(
        &self,
        bitmap: &Bitmap,
        source_x: i32,
        source_y: i32,
        source_width: i32,
        source_height: i32,
        dest_x: i32,
        dest_y: i32,
        dest_width: i32,
        dest_height: i32,
    ) {
        RESOURCES.with(|resources|{
            let _ = self.context2d.draw_image_s(resources.borrow().get(bitmap.url()).unwrap().image.clone(),
            source_x as f64, source_y as f64,
            source_width as f64, source_height as f64,
            dest_x as f64, dest_y as f64,
            dest_width as f64, dest_height as f64);
        });
    }
}

impl SnowballFight {
    pub fn start(resources: &RefMut<HashMap<String, Image>>) {

        let canvas: CanvasElement = document()
            .query_selector("#canvas")
            .unwrap()
            .unwrap()
            .try_into()
            .unwrap();
        canvas.set_width(WIDTH as u32);
        canvas.set_height(HEIGHT as u32);

        //创建并加载人的位图
        let mut person_sprite = PersonSprite::new(Rc::new(RefCell::new(resources.get(IMG_PERSON).unwrap().clone())));
        person_sprite.set_position_point(100.0, 200.0);

        let mut engine = GameEngine::new();
        let player = Rc::new(RefCell::new(person_sprite));
        engine.add_sprite(player.clone());
        let context2d: CanvasRenderingContext2d = canvas.get_context().unwrap();

        GAME.with(|game| {
            *game.borrow_mut() = Some(SnowballFight {
                animation_callback: |timestamp: f64| {
                    GAME.with(|game| {
                        let mut game = game.borrow_mut();
                        let snowball_fight = game.as_mut().unwrap();
                        snowball_fight.animation_frame(timestamp);
                        window().request_animation_frame(snowball_fight.animation_callback);
                    });
                },
                engine,
                last_timestamp: 0.0,
                player,
                context2d,
            });

            //启动游戏循环
            let mut game = game.borrow_mut();
            let snowball_fight = game.as_mut().unwrap();
            window().request_animation_frame(snowball_fight.animation_callback);
        });
    }

    pub fn animation_frame(&mut self, timestamp: f64) {
        self.update(timestamp);
        self.draw();
    }

    pub fn update(&mut self, timestamp: f64) {
        if self.last_timestamp == 0.0 {
            self.last_timestamp = timestamp;
        }
        let elapsed_milis = timestamp - self.last_timestamp;
        self.engine.update_sprites(
            elapsed_milis,
            Rc::new(RefCell::new(SnowballFightUpdateCallback {})),
        );
        self.last_timestamp = timestamp;
    }

    pub fn draw(&self) {
        //绘制子画面
        self.fill_style("#eee");
        self.fill_rect(0, 0, WIDTH, HEIGHT);
        self.engine.draw_sprites(self);
    }

    //走动
    pub fn walk(&mut self, direction: ScrollDir){
        match direction{
            ScrollDir::Right => {
                self.player.borrow_mut().walk();
            }
            _ => {
                self.player.borrow_mut().walk();
            }
        }
    }

    pub fn on_key_down(&mut self, key: &str){
        //console!(log, format!("on_key_down={}, cur_animation_index={}", key, self.player.borrow().cur_animation_index()));
        match key{
            "Left" | "ArrowLeft" => self.walk(ScrollDir::Left),
            "Up" | "ArrowUp" => self.walk(ScrollDir::Up),
            "Down" | "ArrowDown" => self.walk(ScrollDir::Down),
            "Right" | "ArrowRight" => self.walk(ScrollDir::Right),
            "s" => self.player.borrow_mut().crouch(),
            "a" => self.player.borrow_mut().throw_left(),
            "d" => self.player.borrow_mut().throw_right(),
            _ => {}
        }
    }

    pub fn on_key_up(&mut self, key: &str){
        match key{
            "Left" | "ArrowLeft" => self.player.borrow_mut().idle(),
            "Up" | "ArrowUp" => self.player.borrow_mut().idle(),
            "Down" | "ArrowDown" => self.player.borrow_mut().idle(),
            "Right" | "ArrowRight" => self.player.borrow_mut().idle(),
            "s" => self.player.borrow_mut().standup(),
            _ => {}
        }
    }
}

fn load_image_resource(urls: Vec<&str>, callback: fn(String, ImageElement)) {
    for url in urls {
        let cb = move |url: String, image: ImageElement| {
            callback(url, image);
        };
        js!{
            var load_callback = @{cb};
            var onload = function(){
                load_callback(this.url, this);
            };
            var url = @{url};
            var image = new Image();
            image.onload = onload;
            image.url = url;
            image.src = url;
        };
    }
}

fn main() {
    stdweb::initialize();

    load_image_resource(vec![IMG_PERSON], |url, image| {
        RESOURCES.with(|resources| {
            let mut resources = resources.borrow_mut();
            console!(log, format!("加载:{}/1->{}", resources.len() + 1, url));
            resources.insert(url.clone(), Image { url, image });
            if resources.len() == 1 {
                SnowballFight::start(&resources);
            }
        });
    });

    window().add_event_listener(|event: KeyPressEvent| {
        GAME.with(|game| {
            let mut game = game.borrow_mut();
            let snowball_fight = game.as_mut().unwrap();
            snowball_fight.on_key_down(event.key().as_str());
        });
    });
    window().add_event_listener(|event: KeyUpEvent| {
        GAME.with(|game| {
            let mut game = game.borrow_mut();
            let snowball_fight = game.as_mut().unwrap();
            snowball_fight.on_key_up(event.key().as_str());
        });
    });

    stdweb::event_loop();
}
