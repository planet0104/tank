use sprite::BitmapRes;
use canvas::Canvas;
use std::marker::Sized;

pub trait Background{
    fn new(width: i32, height: i32, color: String, bitmap: Option<BitmapRes>) -> Self;

    fn from_bitmap(bitmap: BitmapRes) -> Self where Self: Sized{
        Self::new(
            bitmap.width(),
            bitmap.height(),
            String::new(),
            Some(bitmap))
    }

    fn from_color(width: i32, height: i32, color: String) -> Self where Self: Sized{
        Self::new(
            width,
            height,
            color,
            None)
    }

    fn draw<C: Canvas>(&self, canvas:C);

    fn update(&self);

    fn get_width(&self) ->i32;

    fn get_height(&self)->i32;
}

pub struct StarryBackground{
    width: i32,
    height: i32,
    color: String,
    bitmap: Option<BitmapRes>,
}

impl Background for StarryBackground{
    fn new(width: i32, height: i32, color: String, bitmap: Option<BitmapRes>) -> StarryBackground{
        StarryBackground{
            width,
            height,
            color,
            bitmap
        }
    }

    fn draw<C: Canvas>(&self, canvas:C){

    }

    fn update(&self){

    }

    fn get_width(&self) ->i32{
        self.width
    }

    fn get_height(&self)->i32{
        self.height
    }
}