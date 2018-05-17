pub extern crate engine;

#[macro_use]
extern crate stdweb;
#[macro_use]
extern crate lazy_static;
use engine::utils::rand_int;
use engine::canvas::Canvas;
use engine::{GameEngine, UpdateCallback};
use engine::sprite::{BitmapRes, PointF, Rect, Sprite, BA_DIE, BA_WRAP};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

fn main() {
    println!("Hello, world!");
    
}
