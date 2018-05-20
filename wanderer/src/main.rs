extern crate engine;

#[macro_use]
extern crate stdweb;
use engine::utils::rand_int;
use engine::canvas::Canvas;
use engine::background::{ ScrollingBackground, BackgroundLayer};
use engine::{GameEngine, Bitmap, ScrollDir, UpdateCallback};
use engine::sprite::{Entity, Rect, SPRITEACTION, PointF, Rect, Sprite, BA_DIE, BA_WRAP};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use stdweb::web::html_element::ImageElement;

use stdweb::traits::*;
use stdweb::unstable::TryInto;
use stdweb::web::{
    document,
    window,
    CanvasRenderingContext2d
};

use stdweb::web::event::{
    LoadEndEvent,
    ResourceLoadEvent
};

use stdweb::web::html_element::CanvasElement;

struct PersonSprite{
    entity: Entity,
}

impl PersonSprite{
    pub fn walk(&mut self){
        self.entity.cur_frame = 1 - self.entity.cur_frame;
    }
}

impl Sprite for PersonSprite{
    fn draw(&self, context: &Canvas){
        self.entity.draw(context);
    }
    fn z_order(&self) -> i32{
        self.entity.z_order
    }
    fn position(&self) -> &Rect{
        &self.entity.position
    }
    fn update(&mut self, elapsed_milis: f64) -> SPRITEACTION{
        self.entity.update(elapsed_milis)
    }
    fn id(&self) -> u32{
        self.entity.id
    }
    fn set_position_rect(&mut self, position: Rect){
        self.entity.position = position;
    }
    fn test_collison(&self, test: &Rect) -> bool{
        self.entity.test_collison(test)
    }
    fn kill(&mut self){
        self.entity.dying = true;
    }
    fn class(&self) -> i32{
        0
    }
}

const WIDTH:i32 = 256;
const HEIGHT:i32 = 256;
const IMG_CLOUDS:&'static str = "Background_Clouds.png";
const IMG_LANDSCAPE:&'static str = "Background_Landscape.png";
const IMG_PERSON:&'static str = "Person.png";

thread_local!{
    static GAME: RefCell<Wanderer> = RefCell::new(Wanderer{
        animation_callback:|timestamp: f64|{
            GAME.with(|game|{
                let mut wanderer = game.borrow_mut();
                wanderer.animation_frame(timestamp);
                window().request_animation_frame(wanderer.animation_callback);
            });
        },
        canvas: document().query_selector( "#canvas" ).unwrap().unwrap().try_into().unwrap(),
        resources: HashMap::new(),
        game: None
    });
}

struct Wanderer{
    animation_callback: fn(f64),
    canvas: CanvasElement,
    resources: HashMap<String, Image>,
    game: Option<Game>
}

struct Game{
    background: ScrollingBackground,
    foreground: ScrollingBackground,
    engine: GameEngine
}

struct Image{
    image: ImageElement,
}

impl Bitmap for Image{
    fn width(&self) -> i32{
        self.image.width() as i32
    }
    fn height(&self) -> i32{
        self.image.height() as i32
    }
    fn id(&self) -> u8{
        0
    }
}

impl Wanderer{
    pub fn start(&mut self){
        self.canvas.set_width(WIDTH as u32);
        self.canvas.set_height(HEIGHT as u32);

        //创建滚动背景和风景图层
        let background = ScrollingBackground::new(WIDTH, HEIGHT);
        let bg_landscape_layer = BackgroundLayer::new(
            Box::new(*self.resources.get(IMG_LANDSCAPE).unwrap()),
            0.0,
            ScrollDir::Left
        );
        let viewport = Rect::new(352.0, 352.0, 608.0, 608.0); //视口最初设置为显示风景位图的中央
        bg_landscape_layer.set_viewport(viewport);
        background.add_layer(bg_landscape_layer);

        //创建滚动前景和云彩图层
        let foreground = ScrollingBackground::new(WIDTH, HEIGHT);
        let fg_clouds_layer = BackgroundLayer::new(
            Box::new(*self.resources.get(IMG_CLOUDS).unwrap()),
            0.0,
            ScrollDir::Left
        );
        let viewport = Rect::new(64.0, 64.0, 320.0, 320.0);
        fg_clouds_layer.set_viewport(viewport);
        foreground.add_layer(fg_clouds_layer);

        //创建并加载人的位图
        personBitmap = res.pngPerson;
        var bounds = new Rect(115, 112, 26, 32);
        personSprite = new PersonSprite(personBitmap, bounds, BOUNDS_ACTION_STOP);
        personSprite.setNumFrames(2);
        personSprite.setPosition(115, 112);
        game.addSprite(personSprite);

        window().request_animation_frame(self.animation_callback);
    }

    pub fn animation_frame(&mut self, timestamp: f64){
        //console!(log, "animation_frame 回调.", timestamp);
        let context: CanvasRenderingContext2d = self.canvas.get_context().unwrap();
        let _ = context.draw_image(self.resources.get(IMG_PERSON).unwrap().image.clone(), 0.0, 0.0);
    }
}

fn load_image_resource(urls: Vec<&str>, callback:fn(String, ImageElement)){
    for url in urls{
        let cb = move |url: String, image: ImageElement|{
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

    load_image_resource(vec![
        IMG_CLOUDS,
        IMG_LANDSCAPE,
        IMG_PERSON
    ], |url, image|{
        GAME.with(|game|{
            let mut wanderer = game.borrow_mut();
            console!(log, format!("加载:{}/3->{}", wanderer.resources.len()+1, url));
            wanderer.resources.insert(url, Image{ image });
            if wanderer.resources.len() == 3{
                wanderer.start();
            }
        });
    });

    stdweb::event_loop();
}
