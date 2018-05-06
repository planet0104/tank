use std::ops::{Mul, Sub, AddAssign, SubAssign, DivAssign, MulAssign};

#[derive(Debug)]
pub struct Vector2D{
    pub x: f64,
    pub y: f64,
}

impl Clone for Vector2D {
    fn clone(&self) -> Vector2D {
        Vector2D{ x: self.x, y: self.y }
    }

    fn clone_from(&mut self, source: &Self) {
        self.x = source.x;
        self.y = source.y;
    }
}

// +=
impl AddAssign for Vector2D {
    fn add_assign(&mut self, other: Vector2D) {
         self.x += other.x;
         self.y += other.y;
    }
}

// -=
impl SubAssign for Vector2D {
    fn sub_assign(&mut self, other: Vector2D) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

// *=
impl MulAssign for Vector2D {
    fn mul_assign(&mut self, other: Vector2D) {
        self.x *= other.x;
        self.y *= other.y;
    }
}

// /=
impl DivAssign for Vector2D {
    fn div_assign(&mut self, other: Vector2D) {
        self.x /= other.x;
        self.y /= other.y;
    }
}

//重载 Vector2D*f64
impl Mul<f64> for Vector2D {
    type Output = Vector2D;

    fn mul(self, rhs: f64) -> Vector2D {
        let mut vec = Vector2D::new(self.x, self.y);
        vec.x *= rhs;
        vec.y *= rhs;
        vec
    }
}
//重载 f64*Vector2D
impl Mul<Vector2D> for f64 {
    type Output = Vector2D;

    fn mul(self, rhs: Vector2D) -> Vector2D {
        let vec = Vector2D::new(rhs.x, rhs.y);
        vec * self
    }
}
//重载 Vector2D-Vector2D
impl Sub for Vector2D {
    type Output = Vector2D;

    fn sub(self, other: Vector2D) -> Vector2D {
        let x = self.x - other.x;
        let y = self.y - other.y;
        Vector2D::new(x, y)
    }
}


impl PartialEq for Vector2D {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Vector2D{
    pub fn new(x:f64, y:f64)->Vector2D{
        Vector2D{ x: x, y:y }
    }

    pub fn length(v: &Vector2D)->f64{
        (v.x * v.x + v.y*v.y).sqrt()
    }

    pub fn normalize(v : &mut Vector2D){
        let len = Vector2D::length(v);
        v.x = v.x / len;
        v.y = v.y / len;
    }

    pub fn dot(v1: &Vector2D, v2: &Vector2D)->f64{
        v1.x*v2.x + v1.y*v2.y
    }

    pub fn sign(v1: &Vector2D, v2: &Vector2D)->i32{
        if v1.y*v2.x > v1.x*v2.y {
            return 1;
        }else{
            return -1;
        }
    }

    pub fn sub(v1: &Vector2D, v2: &Vector2D) ->Vector2D {
        Vector2D { x:v1.x - v2.x, y:v1.y - v2.y }
    }

    pub fn mul(v:&Vector2D, d: f64) ->Vector2D{
        Vector2D{ x:v.x * d, y:v.y * d }
    }
}