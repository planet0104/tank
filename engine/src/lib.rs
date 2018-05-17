extern crate rand;
pub mod canvas;
pub mod utils;
pub mod engine;
pub mod sprite;
pub mod vector_2d;

pub use engine::GameEngine as GameEngine;
pub use engine::UpdateCallback as UpdateCallback;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
