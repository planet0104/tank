use engine::Bitmap;
use engine::sprite::{Entity, PointF, Rect, Sprite, BOUNDSACTION};
use stdweb::web::html_element::ImageElement;

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
    pub fn new(bitmap: Box<Bitmap>, bounds: Rect, bounds_action: BOUNDSACTION) -> PersonSprite {
        let mut entity =
            Entity::with_bounds_action(0, bitmap, PointF::zero(), bounds, bounds_action);
        entity.set_num_frames(4, false);
        entity.set_frame_delay(-1);
        PersonSprite { entity }
    }

    pub fn walk(&mut self) {
        self.entity.cur_frame = 1 - self.entity.cur_frame;
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
