extern crate rand;
pub mod background;
pub mod canvas;
pub mod engine;
pub mod sprite;
pub mod utils;
pub mod vector_2d;

pub use engine::GameEngine;
pub use engine::UpdateCallback;

pub trait Bitmap {
    fn width(&self) -> i32;
    fn height(&self) -> i32;
    fn id(&self) -> u8;
}

#[derive(Copy)]
pub struct HtmlImage {
    pub width: i32,
    pub height: i32,
    pub id: u8,
}

impl Clone for HtmlImage {
    fn clone(&self) -> HtmlImage {
        *self
    }
}

impl HtmlImage {
    pub fn new(id: u8, width: i32, height: i32) -> HtmlImage {
        HtmlImage { width, height, id }
    }
}

impl Bitmap for HtmlImage {
    fn width(&self) -> i32 {
        self.width
    }
    fn height(&self) -> i32 {
        self.height
    }
    fn id(&self) -> u8 {
        self.id
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
