//画布
pub trait Canvas {
    fn draw_image_repeat(&self, res_id: i32, x: i32, y: i32, width: i32, height: i32);
    fn draw_image_repeat_x(&self, res_id: i32, x: i32, y: i32, width: i32, height: i32);
    fn draw_image_repeat_y(&self, res_id: i32, x: i32, y: i32, width: i32, height: i32);
    fn draw_image_at(&self, res_id: i32, x: i32, y: i32);
    fn draw_image(
        &self,
        res_id: i32,
        source_x: i32,
        source_y: i32,
        source_width: i32,
        source_height: i32,
        dest_x: i32,
        dest_y: i32,
        dest_width: i32,
        dest_height: i32,
    );
    fn line_width(&self, width: i32);
    fn set_font(&self, font: &str);
    fn fill_style(&self, style: &str);
    fn stroke_style(&self, style: &str);
    fn fill_rect(&self, x: i32, y: i32, width: i32, height: i32);
    fn stroke_rect(&self, x: i32, y: i32, width: i32, height: i32);
    fn fill_text(&self, text: &str, x: i32, y: i32);
}