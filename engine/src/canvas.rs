use Bitmap;

//画布
pub trait Canvas {
    fn draw_image_repeat(&self, _image: &Bitmap, _x: i32, _y: i32, _width: i32, _height: i32) {}
    fn draw_image_repeat_x(&self, _image: &Bitmap, _x: i32, _y: i32, _width: i32, _height: i32) {}
    fn draw_image_repeat_y(&self, _image: &Bitmap, _x: i32, _y: i32, _width: i32, _height: i32) {}
    fn draw_image_at(&self, image: &Bitmap, x: i32, y: i32);
    fn draw_image(
        &self,
        image: &Bitmap,
        source_x: i32,
        source_y: i32,
        source_width: i32,
        source_height: i32,
        dest_x: i32,
        dest_y: i32,
        dest_width: i32,
        dest_height: i32,
    );
    fn translate(&self, _x: f64, _y:f64){}
    fn scale(&self, _x: f64, _y:f64){}
    fn rotate(&self, _degree: f64){}
    fn save(&self){}
    fn restore(&self){}
    fn line_width(&self, _width: i32) {}
    fn set_font(&self, _font: &str) {}
    fn fill_style(&self, _style: &str) {}
    fn stroke_style(&self, _style: &str) {}
    fn fill_rect(&self, _x: i32, _y: i32, _width: i32, _height: i32) {}
    fn stroke_rect(&self, _x: i32, _y: i32, _width: i32, _height: i32) {}
    fn fill_text(&self, _text: &str, _x: i32, _y: i32) {}
    fn console_log(&self, _s: &str) {}
}
