use engine::Bitmap;
use engine::sprite::{Entity, PointF, Rect, Sprite, BA_STOP};
use engine::animation::Animation;
use stdweb::web::html_element::ImageElement;
use std::rc::Rc;
use std::cell::RefCell;
use {WIDTH, HEIGHT};

pub struct Image {
    pub image: ImageElement,
    pub url: String,
}

impl Bitmap for Image {
    fn width(&self) -> i32 {
        self.image.width() as i32
    }
    fn height(&self) -> i32 {
        self.image.height() as i32
    }
    fn id(&self) -> u8 {
        0
    }
    fn url(&self) -> &str {
        &self.url
    }
}

impl Clone for Image {
    fn clone(&self) -> Image {
        Image {
            image: self.image.clone(),
            url: self.url.clone(),
        }
    }
}

pub struct PersonSprite {
    entity: Entity,
}

impl PersonSprite {
    pub fn new(bitmap: Rc<RefCell<Bitmap>>) -> PersonSprite {
        let entity = Entity::new(
            0,
            vec![
                Animation::infinite(bitmap, 0, 0, 167, 220, 15, 500)
            ],
            PointF::zero(),
            Rect::new(0.0, 0.0, WIDTH as f64, HEIGHT as f64),
            BA_STOP
        );
        PersonSprite { entity }
    }

    pub fn walk(&mut self) {
        //self.entity.cur_frame = 1 - self.entity.cur_frame;
    }
}

impl Sprite for PersonSprite {
    fn class(&self) -> i32 {
        0
    }
    fn get_entity(&self) -> &Entity {
        &self.entity
    }
    fn get_entity_mut(&mut self) -> &mut Entity {
        &mut self.entity
    }
}
