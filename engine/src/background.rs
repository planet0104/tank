use Bitmap;
use canvas::Canvas;
use sprite::Rect;

pub trait Background {
    fn draw<C: Canvas>(&self, canvas: &C);
    fn update(&mut self);
    fn width(&self) -> i32;
    fn height(&self) -> i32;
}

pub struct ScrollingBackground {
    layers: Vec<BackgroundLayer>,
    width: i32,
    height: i32,
}

impl ScrollingBackground {
    pub fn new(width: i32, height: i32) -> ScrollingBackground {
        ScrollingBackground {
            width,
            height,
            layers: vec![],
        }
    }

    pub fn add_layer(&mut self, layer: BackgroundLayer){
        self.layers.push(layer);
    }
}

impl Background for ScrollingBackground {
    fn draw<C: Canvas>(&self, canvas: &C) {
        for layer in &self.layers {
            layer.draw(canvas, 0, 0)
        }
    }

    fn update(&mut self) {
        //更新图层
        for layer in &mut self.layers {
            layer.update();
        }
    }

    fn width(&self) -> i32 {
        self.width
    }

    fn height(&self) -> i32 {
        self.height
    }
}

pub enum ScrollDir {
    Up,
    Right,
    Down,
    Left,
}

pub struct BackgroundLayer {
    viewport: Rect,
    speed: f64,
    direction: ScrollDir,
    bitmap: Box<Bitmap>,
}

impl BackgroundLayer {
    pub fn new(bitmap: Box<Bitmap>, speed: f64, direction: ScrollDir) -> BackgroundLayer {
        let viewport = Rect::new(0.0, 0.0, bitmap.width() as f64, bitmap.height() as f64);
        BackgroundLayer {
            speed,
            direction,
            bitmap,
            viewport,
        }
    }

    pub fn update(&mut self) {
        match self.direction {
            ScrollDir::Up => {
                // Move the layer up (slide the viewport down)
                self.viewport.top += self.speed;
                self.viewport.bottom += self.speed;
                if self.viewport.top > self.height() as f64 {
                    self.viewport.bottom = self.viewport.bottom - self.viewport.top;
                    self.viewport.top = 0.0;
                }
            }

            ScrollDir::Right => {
                // Move the layer right (slide the viewport left)
                self.viewport.left -= self.speed;
                self.viewport.right -= self.speed;
                if self.viewport.right < 0.0 {
                    self.viewport.left =
                        self.width() as f64 - (self.viewport.right - self.viewport.left);
                    self.viewport.right = self.width() as f64;
                }
            }

            ScrollDir::Down => {
                // Move the layer down (slide the viewport up)
                self.viewport.top -= self.speed;
                self.viewport.bottom -= self.speed;
                if self.viewport.bottom < 0.0 {
                    self.viewport.top =
                        self.height() as f64 - (self.viewport.bottom - self.viewport.top);
                    self.viewport.bottom = self.height() as f64;
                }
            }

            ScrollDir::Left => {
                // Move the layer left (slide the viewport right)
                self.viewport.left += self.speed;
                self.viewport.right += self.speed;
                if self.viewport.left > self.width() as f64 {
                    self.viewport.right = self.viewport.right - self.viewport.left;
                    self.viewport.left = 0.0;
                }
            }
        }
    }

    pub fn draw<C: Canvas>(&self, canvas: &C, x: i32, y: i32) {
        //仅绘制通过视口看到的图层部分
        if self.viewport.top < 0.0 && self.viewport.left < 0.0 {
            //绘制分割视口，从上到下，从左到右
            //绘制左上部分(对应图片右下部分)
            canvas.draw_image(
                self.bitmap.as_ref(),
                self.width() + self.viewport.left as i32 as i32,
                self.height() + self.viewport.top as i32, //图像源左上角
                -self.viewport.left as i32,
                -self.viewport.top as i32, //图像源宽高
                x,
                y, //目标绘制坐标
                -self.viewport.left as i32,
                -self.viewport.top as i32,
            );
            //绘制右上部分(对应图片左下部分)
            canvas.draw_image(
                self.bitmap.as_ref(),
                0,
                self.height() + self.viewport.top as i32,
                -self.viewport.right as i32,
                -self.viewport.top as i32,
                x - self.viewport.left as i32,
                y,
                -self.viewport.right as i32,
                -self.viewport.top as i32,
            );
            //绘制左下部分(对应图片右上部分)
            canvas.draw_image(
                self.bitmap.as_ref(),
                self.width() + self.viewport.left as i32,
                0,
                -self.viewport.left as i32,
                self.viewport.bottom as i32,
                x,
                y - self.viewport.top as i32,
                -self.viewport.left as i32,
                self.viewport.bottom as i32,
            );
            //绘制右下部分(对应图片左上部分)
            canvas.draw_image(
                self.bitmap.as_ref(),
                0,
                0,
                self.viewport.right as i32,
                self.viewport.bottom as i32,
                x - self.viewport.left as i32,
                y - self.viewport.top as i32,
                self.viewport.right as i32,
                self.viewport.bottom as i32,
            );
        } else if self.viewport.top < 0.0 && self.viewport.right as i32 > self.width() {
            //绘制拆开的视口，从顶部环绕到底部，从右侧环绕到左侧
            canvas.draw_image(
                self.bitmap.as_ref(),
                self.viewport.left as i32,
                self.height() + self.viewport.top as i32,
                self.width() - self.viewport.left as i32,
                -self.viewport.top as i32,
                x,
                y,
                self.width() - self.viewport.left as i32,
                -self.viewport.top as i32,
            );
            canvas.draw_image(
                self.bitmap.as_ref(),
                0,
                self.height() + self.viewport.top as i32,
                self.viewport.right as i32 - self.width(),
                -self.viewport.top as i32,
                x + (self.width() - self.viewport.left as i32),
                y,
                self.viewport.right as i32 - self.width(),
                -self.viewport.top as i32,
            );
            canvas.draw_image(
                self.bitmap.as_ref(),
                self.viewport.left as i32,
                0,
                self.width() - self.viewport.left as i32,
                self.viewport.bottom as i32,
                x,
                y - self.viewport.top as i32,
                self.width() - self.viewport.left as i32,
                self.viewport.bottom as i32,
            );
            canvas.draw_image(
                self.bitmap.as_ref(),
                0,
                0,
                self.viewport.right as i32 - self.width(),
                self.viewport.bottom as i32,
                x + (self.width() - self.viewport.left as i32),
                y - self.viewport.top as i32,
                self.viewport.right as i32 - self.width(),
                self.viewport.bottom as i32,
            );
        } else if self.viewport.bottom as i32 > self.height() && self.viewport.left < 0.0 {
            //绘制拆开的视口，从底部环绕到顶部，从左侧环绕到右侧
            canvas.draw_image(
                self.bitmap.as_ref(),
                self.width() + self.viewport.left as i32,
                self.viewport.top as i32,
                -self.viewport.left as i32,
                self.height() - self.viewport.top as i32,
                x,
                y,
                -self.viewport.left as i32,
                self.height() - self.viewport.top as i32,
            );
            canvas.draw_image(
                self.bitmap.as_ref(),
                0,
                self.viewport.top as i32,
                self.viewport.right as i32,
                self.height() - self.viewport.top as i32,
                x - self.viewport.left as i32,
                y,
                self.viewport.right as i32,
                self.height() - self.viewport.top as i32,
            );
            canvas.draw_image(
                self.bitmap.as_ref(),
                self.width() + self.viewport.left as i32,
                0,
                -self.viewport.left as i32,
                self.viewport.bottom as i32 - self.height(),
                x,
                y + (self.height() - self.viewport.top as i32),
                -self.viewport.left as i32,
                self.viewport.bottom as i32 - self.height(),
            );
            canvas.draw_image(
                self.bitmap.as_ref(),
                0,
                0,
                self.viewport.right as i32,
                self.viewport.bottom as i32 - self.height(),
                x - self.viewport.left as i32,
                y + (self.height() - self.viewport.top as i32),
                self.viewport.right as i32,
                self.viewport.bottom as i32 - self.height(),
            );
        } else if self.viewport.bottom as i32 > self.height()
            && self.viewport.right as i32 > self.width()
        {
            //绘制所有窗口，从底部环绕到顶部，从右侧环绕到左侧
            canvas.draw_image(
                self.bitmap.as_ref(),
                self.viewport.left as i32,
                self.viewport.top as i32,
                self.width() - self.viewport.left as i32,
                self.height() - self.viewport.top as i32,
                x,
                y,
                self.width() - self.viewport.left as i32,
                self.height() - self.viewport.top as i32,
            );
            canvas.draw_image(
                self.bitmap.as_ref(),
                0,
                self.viewport.top as i32,
                self.viewport.right as i32 - self.width(),
                self.height() - self.viewport.top as i32,
                x + (self.width() - self.viewport.left as i32),
                y,
                self.viewport.right as i32 - self.width(),
                self.height() - self.viewport.top as i32,
            );
            canvas.draw_image(
                self.bitmap.as_ref(),
                self.viewport.left as i32,
                0,
                self.width() - self.viewport.left as i32,
                self.viewport.bottom as i32 - self.height(),
                x,
                y + (self.height() - self.viewport.top as i32),
                self.width() - self.viewport.left as i32,
                self.viewport.bottom as i32 - self.height(),
            );
            canvas.draw_image(
                self.bitmap.as_ref(),
                0,
                0,
                self.viewport.right as i32 - self.width(),
                self.viewport.bottom as i32 - self.height(),
                x + (self.width() - self.viewport.left as i32),
                y + (self.height() - self.viewport.top as i32),
                self.viewport.right as i32 - self.width(),
                self.viewport.bottom as i32 - self.height(),
            );
        } else if self.viewport.top < 0.0 {
            //绘制拆开的视口，从顶部环绕到底部
            canvas.draw_image(
                self.bitmap.as_ref(),
                self.viewport.left as i32,
                self.height() + self.viewport.top as i32,
                self.viewport.right as i32 - self.viewport.left as i32,
                -self.viewport.top as i32,
                x,
                y,
                self.viewport.right as i32 - self.viewport.left as i32,
                -self.viewport.top as i32,
            );
            canvas.draw_image(
                self.bitmap.as_ref(),
                self.viewport.left as i32,
                0,
                self.viewport.right as i32 - self.viewport.left as i32,
                self.viewport.bottom as i32,
                x,
                y - self.viewport.top as i32,
                self.viewport.right as i32 - self.viewport.left as i32,
                self.viewport.bottom as i32,
            );
        } else if self.viewport.right as i32 > self.width() {
            //绘制拆开的视口，从右侧环绕到左侧
            let w = self.width() - self.viewport.left as i32;
            let h = self.viewport.bottom as i32 - self.viewport.top as i32;
            if w > 0 && h > 0 {
                canvas.draw_image(
                    self.bitmap.as_ref(),
                    self.viewport.left as i32,
                    self.viewport.top as i32,
                    w,
                    h,
                    x,
                    y,
                    w,
                    h,
                );
            }

            canvas.draw_image(
                self.bitmap.as_ref(),
                0,
                self.viewport.top as i32,
                self.viewport.right as i32 - self.width(),
                self.viewport.bottom as i32 - self.viewport.top as i32,
                x + (self.width() - self.viewport.left as i32),
                y,
                self.viewport.right as i32 - self.width(),
                self.viewport.bottom as i32 - self.viewport.top as i32,
            );
        } else if self.viewport.bottom as i32 > self.height() {
            //绘制拆开的窗口，从底部环绕到顶部
            canvas.draw_image(
                self.bitmap.as_ref(),
                self.viewport.left as i32,
                self.viewport.top as i32,
                self.viewport.right as i32 - self.viewport.left as i32,
                self.height() - self.viewport.top as i32,
                x,
                y,
                self.viewport.right as i32 - self.viewport.left as i32,
                self.height() - self.viewport.top as i32,
            );
            canvas.draw_image(
                self.bitmap.as_ref(),
                self.viewport.left as i32,
                0,
                self.viewport.right as i32 - self.viewport.left as i32,
                self.viewport.bottom as i32 - self.height(),
                x,
                y + (self.height() - self.viewport.top as i32),
                self.viewport.right as i32 - self.viewport.left as i32,
                self.viewport.bottom as i32 - self.height(),
            );
        } else if self.viewport.left < 0.0 {
            //绘制拆开的视口，从左侧环绕到右侧
            canvas.draw_image(
                self.bitmap.as_ref(),
                self.width() + self.viewport.left as i32,
                self.viewport.top as i32,
                -self.viewport.left as i32,
                self.viewport.bottom as i32 - self.viewport.top as i32,
                x,
                y,
                -self.viewport.left as i32,
                self.viewport.bottom as i32 - self.viewport.top as i32,
            );
            canvas.draw_image(
                self.bitmap.as_ref(),
                0,
                self.viewport.top as i32,
                self.viewport.right as i32,
                self.viewport.bottom as i32 - self.viewport.top as i32,
                x - self.viewport.left as i32,
                y,
                self.viewport.right as i32,
                self.viewport.bottom as i32 - self.viewport.top as i32,
            );
        } else {
            //一次性绘制整个视口
            canvas.draw_image(
                self.bitmap.as_ref(),
                self.viewport.left as i32,
                self.viewport.top as i32,
                self.viewport.right as i32 - self.viewport.left as i32,
                self.viewport.bottom as i32 - self.viewport.top as i32,
                x,
                y,
                self.viewport.right as i32 - self.viewport.left as i32,
                self.viewport.bottom as i32 - self.viewport.top as i32,
            );
        }
    }

    pub fn set_speed(&mut self, speed: f64) {
        self.speed = speed;
    }

    pub fn set_direction(&mut self, direction: ScrollDir) {
        self.direction = direction;
    }

    pub fn set_viewport(&mut self, viewport: Rect) {
        self.viewport = viewport;
    }

    pub fn width(&self) -> i32 {
        self.bitmap.width()
    }

    pub fn height(&self) -> i32 {
        self.bitmap.height()
    }
}
