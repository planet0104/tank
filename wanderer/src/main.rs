extern crate engine;

#[macro_use]
extern crate stdweb;

mod sprite;
use engine::background::{Background, BackgroundLayer, ScrollDir, ScrollingBackground};
use engine::canvas::Canvas;
use engine::sprite::{Rect, Sprite, BA_STOP};
use engine::{Bitmap, GameEngine, UpdateCallback};
use sprite::{Image, PersonSprite};
use std::cell::RefCell;
use std::cell::RefMut;
use std::collections::HashMap;
use std::rc::Rc;
use stdweb::web::html_element::ImageElement;

use stdweb::traits::*;
use stdweb::unstable::TryInto;
use stdweb::web::{document, window, CanvasRenderingContext2d};
use stdweb::web::event::{
    KeyPressEvent
};
use stdweb::web::html_element::CanvasElement;

const WIDTH: i32 = 256;
const HEIGHT: i32 = 256;
const IMG_CLOUDS: &'static str = "Background_Clouds.png";
const IMG_LANDSCAPE: &'static str = "Background_Landscape.png";
const IMG_PERSON: &'static str = "Person.png";

thread_local!{
    static GAME: RefCell<Option<Wanderer>> = RefCell::new(None);
    static RESOURCES: RefCell<HashMap<String, Image>> = RefCell::new(HashMap::new());
}

//客户端游戏更新(不做任何处理)
pub struct WandererUpdateCallback {}
impl UpdateCallback for WandererUpdateCallback {
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

struct Wanderer {
    animation_callback: fn(f64),
    context2d: CanvasRenderingContext2d,
    background: ScrollingBackground,
    bg_landscape_layer: Rc<RefCell<BackgroundLayer>>,
    foreground: ScrollingBackground,
    fg_clouds_layer:  Rc<RefCell<BackgroundLayer>>,
    engine: GameEngine,
    player: Rc<RefCell<PersonSprite>>,
    last_timestamp: f64,
}

impl Canvas for Wanderer {
    fn draw_image_at(&self, bitmap: &Bitmap, x: i32, y: i32) {
        RESOURCES.with(|resources|{
            self.context2d.draw_image(resources.borrow().get(bitmap.url()).unwrap().image.clone(), x as f64, y as f64).unwrap();
        });
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

impl Wanderer {
    pub fn start(resources: &RefMut<HashMap<String, Image>>) {
        let canvas: CanvasElement = document()
            .query_selector("#canvas")
            .unwrap()
            .unwrap()
            .try_into()
            .unwrap();
        canvas.set_width(WIDTH as u32);
        canvas.set_height(HEIGHT as u32);

        //创建滚动背景和风景图层
        let mut background = ScrollingBackground::new(WIDTH, HEIGHT);
        let mut bg_landscape_layer = BackgroundLayer::new(
            Box::new(resources.get(IMG_LANDSCAPE).unwrap().clone()),
            0.0,
            ScrollDir::Left,
        );
        let viewport = Rect::new(352.0, 352.0, 608.0, 608.0); //视口最初设置为显示风景位图的中央
        bg_landscape_layer.set_viewport(viewport);
        let bg_landscape_layer = Rc::new(RefCell::new(bg_landscape_layer));
        background.add_layer(bg_landscape_layer.clone());

        //创建滚动前景和云彩图层
        let mut foreground = ScrollingBackground::new(WIDTH, HEIGHT);
        let mut fg_clouds_layer = BackgroundLayer::new(
            Box::new(resources.get(IMG_CLOUDS).unwrap().clone()),
            0.0,
            ScrollDir::Left,
        );
        let viewport = Rect::new(64.0, 64.0, 320.0, 320.0);
        fg_clouds_layer.set_viewport(viewport);
        let fg_clouds_layer = Rc::new(RefCell::new(fg_clouds_layer));
        foreground.add_layer(fg_clouds_layer.clone());

        //创建并加载人的位图
        let person_bitmap = Box::new(resources.get(IMG_PERSON).unwrap().clone());
        let bounds = Rect::new(115.0, 112.0, 26.0, 32.0);
        let mut person_sprite = PersonSprite::new(person_bitmap, bounds, BA_STOP);
        person_sprite.set_position_point(115.0, 112.0);
        person_sprite.get_entity_mut().num_frames = 2;

        let mut engine = GameEngine::new();
        let player = Rc::new(RefCell::new(person_sprite));
        engine.add_sprite(player.clone());
        let context2d: CanvasRenderingContext2d = canvas.get_context().unwrap();

        GAME.with(|game| {
            *game.borrow_mut() = Some(Wanderer {
                animation_callback: |timestamp: f64| {
                    GAME.with(|game| {
                        let mut game = game.borrow_mut();
                        let wanderer = game.as_mut().unwrap();
                        wanderer.animation_frame(timestamp);
                        window().request_animation_frame(wanderer.animation_callback);
                    });
                },
                background,
                foreground,
                engine,
                last_timestamp: 0.0,
                player,
                context2d,
                bg_landscape_layer,
                fg_clouds_layer
            });

            //启动游戏循环
            let mut game = game.borrow_mut();
            let wanderer = game.as_mut().unwrap();
            window().request_animation_frame(wanderer.animation_callback);
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
            Rc::new(RefCell::new(WandererUpdateCallback {})),
        );
    }

    pub fn draw(&self) {
        //绘制滚动背景
        self.background.draw(self);
        //绘制子画面
        self.engine.draw_sprites(self);
        //绘制滚动背景
        self.foreground.draw(self);
    }

    pub fn drive(&mut self, direction: ScrollDir){
        //让人走动
        self.player.borrow_mut().walk();
        //向右移动风景图层
        let mut bg_landscape_layer = self.bg_landscape_layer.borrow_mut();
        bg_landscape_layer.set_speed(16.0);
        bg_landscape_layer.set_direction(direction);
        bg_landscape_layer.update();
        bg_landscape_layer.set_speed(0.0);

        //向右移动云彩图层
        let mut fg_clouds_layer = self.fg_clouds_layer.borrow_mut();
        fg_clouds_layer.set_speed(4.0);
        fg_clouds_layer.set_direction(direction);
        fg_clouds_layer.update();
        fg_clouds_layer.set_speed(0.0);
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

    load_image_resource(vec![IMG_CLOUDS, IMG_LANDSCAPE, IMG_PERSON], |url, image| {
        RESOURCES.with(|resources| {
            let mut resources = resources.borrow_mut();
            console!(log, format!("加载:{}/3->{}", resources.len() + 1, url));
            resources.insert(url.clone(), Image { url, image });
            if resources.len() == 3 {
                Wanderer::start(&resources);
            }
        });
    });

    window().add_event_listener(|event: KeyPressEvent| {
        GAME.with(|game| {
            let mut game = game.borrow_mut();
            let wanderer = game.as_mut().unwrap();
            match event.key().as_str(){
                "Left" | "ArrowLeft" => wanderer.drive(ScrollDir::Left),
                "Up" | "ArrowUp" => wanderer.drive(ScrollDir::Up),
                "Down" | "ArrowDown" => wanderer.drive(ScrollDir::Down),
                "Right" | "ArrowRight" => wanderer.drive(ScrollDir::Right),
                _ => {}
            }; 
        });
    });

    stdweb::event_loop();
}
