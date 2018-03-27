use engine::{GameEngine, GameEngineHandler};
use background::StarryBackground;
use std::cmp;
use sprite::{Sprite, Point, Rect, BA_BOUNCE, BA_DIE, BA_WRAP, BitmapRes};
use alien_sprite::AlienSprite;
use ::{window, rand_int, draw_image_at, draw_text, fill_style_rgb};

//--------------------------------------------
//-------------游戏资源ID----------------------
//--------------------------------------------
pub const RES_SPLASH_BITMAP:i32 = 0;
pub const RES_DESERT_BITMAP:i32 = 1;
pub const RES_CAR_BITMAP:i32 = 2;
pub const RES_SM_CAR_BITMAP:i32 = 3;
pub const RES_MISSILE_BITMAP:i32 = 4;
pub const RES_BLOBBO_BITMAP:i32   = 5;
pub const RES_BMISSILE_BITMAP:i32 = 6;
pub const RES_JELLY_BITMAP:i32    = 7;
pub const RES_JMISSILE_BITMAP:i32 = 8;
pub const RES_TIMMY_BITMAP:i32    = 9;
pub const RES_TMISSILE_BITMAP:i32 = 10;
pub const RES_SM_EXPLOSION_BITMAP:i32 = 11;
pub const RES_LG_EXPLOSION_BITMAP:i32 = 12;
pub const RES_GAME_OVER_BITMAP:i32 = 13;

pub const RES_BMISSILE_SOUND:i32 = 100;
pub const RES_GAMEOVER_SOUND:i32 = 101;
pub const RES_JMISSILE_SOUND:i32 = 102;
pub const RES_LG_EXPLODE_SOUND:i32 = 103;
pub const RES_SM_EXPLODE_SOUND:i32 = 104;
pub const RES_MISSILE_SOUND:i32 = 105;
pub const URL_BACKGROUND_MUSIC:&str = "Music.mp3";

//-----------------------------------
//-------------事件ID----------------
pub const EVENT_MOUSE_MOVE:i32 = 0;
pub const EVENT_MOUSE_CLICK:i32 = 1;
pub const EVENT_TOUCH_MOVE:i32 = 10;

pub const CLIENT_WIDTH:i32 = 600;
pub const CLIENT_HEIGHT:i32 = 450;

//SpaceOut游戏主结构体
pub struct SpaceOut{
    engine: GameEngine,
    difficulty:i32,
    game_over:bool,
    demo: bool,
    background:StarryBackground,
    game_over_delay:i32,
    fire_input_delay:i32,
    score:i32,
    num_lives:i32,
    car_sprite_id:f64,
    _last_touch: Option<Point>,
    _drive_left:i32,
    _drive_right:i32,
}

impl SpaceOut{
    pub fn new()->SpaceOut{
        SpaceOut{
            engine: GameEngine::new(30, CLIENT_WIDTH, CLIENT_HEIGHT, GameHandler{}),
            difficulty: 80,
            game_over: false,
            demo: true,
            background: StarryBackground::default(CLIENT_WIDTH, CLIENT_HEIGHT),
            game_over_delay: 0,
            fire_input_delay: 0,
            score: 0,
            num_lives: 3,
            car_sprite_id: 0.0,
            _last_touch: None,
            _drive_left: 0,
            _drive_right: 0,
        }
    }

    //新游戏
    pub fn new_game(&mut self){
        //清除所有精灵
        self.engine.clean_up_sprites();
        //初始化游戏变量
        self.fire_input_delay = 0;
        self.score = 0;
        self.num_lives = 3;
        self.difficulty = 80;
        self.game_over = false;

        if self.demo{
            //添加一些外星人
            for _ in 0..6{
                self.add_alien();
            }
        }else{
            //创建汽车
            let mut car_sprite = Sprite::with_bounds_action(
                                BitmapRes::new(RES_CAR_BITMAP, 37, 18),
                                Rect::new(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT), BA_WRAP);
            self.car_sprite_id = car_sprite.id();
            car_sprite.set_position(300, 405);

            self.engine.add_sprite(car_sprite);
            GameEngine::play_music(URL_BACKGROUND_MUSIC);
        }
    }

    //添加外星人
    pub fn add_alien(&mut self){
        //创建一个随机的外星人精灵
        let bounds = Rect::new(0, 0, CLIENT_WIDTH, 410);
        self.engine.add_sprite(match rand_int(0, 3){
            1 => {
                // Blobbo
                let mut sprite = Sprite::with_bounds_action(
                                BitmapRes::new(RES_BLOBBO_BITMAP, 32, 272),
                                bounds, BA_BOUNCE);
                sprite.set_num_frames(8, false);
                sprite.set_position(match rand_int(0, 2){
                    0=>0,
                    _=>600
                }, rand_int(0, 370));
                sprite.ext(AlienSprite{});
                sprite.set_velocity(rand_int(0, 7)-2, rand_int(0, 7)-2);
                sprite
            },
            2 => {
                // Jelly
                let mut sprite = Sprite::with_bounds_action(
                                BitmapRes::new(RES_JELLY_BITMAP, 33, 264),
                                bounds, BA_BOUNCE);
                sprite.set_num_frames(8, false);
                sprite.set_position(rand_int(0, CLIENT_WIDTH), rand_int(0, 370));
                sprite.set_velocity(rand_int(0, 5)-2, rand_int(0, 5)+3);
                sprite.ext(AlienSprite{});
                sprite
            }
            _ =>{
                // Timmy
                let mut sprite = Sprite::with_bounds_action(
                                BitmapRes::new(RES_TIMMY_BITMAP, 33, 136),
                                bounds, BA_WRAP);
                sprite.set_num_frames(8, false);
                sprite.set_position(rand_int(0, CLIENT_WIDTH), rand_int(0, 370));
                sprite.set_velocity(rand_int(0, 7)+3, 0);
                sprite.ext(AlienSprite{});
                sprite
            }
        });
    }

    //游戏循环
    pub fn game_cycle(&mut self){
        if !self.game_over {
            if !self.demo {
                // 随机添加外星人
                if rand_int(0, self.difficulty/2) == 0{
                    self.add_alien();
                }
            }
            //更新背景图
            self.background.update();

            //更新精灵
            self.engine.update_sprites();
        }else{
            self.game_over_delay -= 1;
            if self.game_over_delay == 0{
                //停止播放背景音乐，转换到演示模式
                GameEngine::pause_music();
                self.demo = true;
                self.new_game();
            }
        }
        //绘制游戏
        self.game_paint();
    }

    //游戏绘制
    pub fn game_paint(&self){
        //绘制背景
        self.background.draw();

        //绘制沙漠
        draw_image_at(RES_DESERT_BITMAP, 0, 371);

        //绘制精灵
        self.engine.draw_sprites();

        if self.demo{
            //绘制闪屏图片
            draw_image_at(RES_SPLASH_BITMAP, 142, 20);

            //绘制控制说明
            fill_style_rgb(255, 255, 255);
            draw_text("点击屏幕->发射导弹", 220, 300);
            draw_text("       左滑->倒车", 220, 330);
            draw_text("       右滑->前进", 220, 360);
        }else{
            //绘制得分
            fill_style_rgb(255, 255, 255);
            draw_text(format!("得分：{}", self.score).as_str(), 260, 90);

            //绘制剩余生命
            for i in 0..self.num_lives{
                draw_image_at(RES_SM_CAR_BITMAP, 520+25*i, 10);
            }
            if self.game_over{
                draw_image_at(RES_GAME_OVER_BITMAP, 170, 100);
            }
        }
    }

    //碰撞检测
    pub fn sprite_collision(&mut self, sprite_hitter:&Sprite, sprite_hittee:&Sprite)->bool{
        //检查是否玩家的子弹和外星人相撞
        let hitter = sprite_hitter.bitmap().id();
        let hittee = sprite_hittee.bitmap().id();
        if hitter == RES_MISSILE_BITMAP && (hittee == RES_BLOBBO_BITMAP ||
            hittee == RES_JELLY_BITMAP || hittee == RES_TIMMY_BITMAP) ||
            hittee == RES_MISSILE_BITMAP && (hitter == RES_BLOBBO_BITMAP ||
            hitter == RES_JELLY_BITMAP || hitter == RES_TIMMY_BITMAP){
            //播放小的爆炸声音
            GameEngine::play_sound(RES_SM_EXPLODE_SOUND);
            //杀死子弹和外星人
            self.engine.kill_sprite(sprite_hitter);
            self.engine.kill_sprite(sprite_hittee);

            //在外星人位置创建一个大的爆炸精灵
            let pos:&Rect = if hitter == RES_MISSILE_BITMAP{
                sprite_hittee.position()
            }else{
                sprite_hitter.position()
            };
            let mut sprite = Sprite::from_bitmap(
                BitmapRes::new(RES_LG_EXPLOSION_BITMAP, 33, 272),
                Rect::new(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT));
            sprite.set_num_frames(8, true);
            sprite.set_position(pos.left, pos.top);
            self.engine.add_sprite(sprite);

            //更新得分
            self.score += 25;
            self.difficulty = cmp::max(80-(self.score/20), 20);
        }
        //检查是否有外星人子弹撞到汽车
        if hitter == RES_CAR_BITMAP && (hittee == RES_BMISSILE_BITMAP ||
            hittee == RES_JMISSILE_BITMAP || hittee == RES_TMISSILE_BITMAP) ||
            hittee == RES_CAR_BITMAP && (hitter == RES_BMISSILE_BITMAP ||
            hitter == RES_JMISSILE_BITMAP || hitter == RES_TMISSILE_BITMAP){
            //播放大的爆炸声音
            GameEngine::play_sound(RES_LG_EXPLODE_SOUND);
            //杀死子弹精灵
            if hitter == RES_CAR_BITMAP{
                self.engine.kill_sprite(sprite_hittee);
            }else{
                self.engine.kill_sprite(sprite_hitter);
            }

            //在汽车位置创建一个大的爆炸精灵
            let pos:&Rect = if hitter == RES_CAR_BITMAP{
                sprite_hitter.position()
            }else{
                sprite_hittee.position()
            };
            let mut sprite = Sprite::from_bitmap(
                BitmapRes::new(RES_LG_EXPLOSION_BITMAP, 33, 272),
                Rect::new(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT));
            sprite.set_num_frames(8, true);
            sprite.set_position(pos.left, pos.top);
            self.engine.add_sprite(sprite);

            //移动汽车到起点
            self.engine.get_sprite(self.car_sprite_id).unwrap().set_position(300, 405);
            self.num_lives -= 1;

            //检查游戏是否结束
            if self.num_lives == 0{
                //播放游戏结束声音
                GameEngine::play_sound(RES_GAMEOVER_SOUND);
                self.game_over = true;
                self.game_over_delay = 150;
            }
        }
        false
    }

    //精灵死亡处理
    pub fn sprite_dying(&mut self, sprite_dying:&Sprite){
        //检查是否子弹精灵死亡
        let bitmap_id = sprite_dying.bitmap().id();
        if bitmap_id == RES_BMISSILE_BITMAP ||
            bitmap_id == RES_JMISSILE_BITMAP ||
            bitmap_id == RES_TMISSILE_BITMAP{
            //播放小的爆炸声音
            if !self.demo{
                GameEngine::play_sound(RES_SM_EXPLODE_SOUND);
            }
            //在子弹位置创建一个小的爆炸精灵
            let mut sprite = Sprite::from_bitmap(
                BitmapRes::new(RES_SM_EXPLOSION_BITMAP, 17, 136),
                Rect::new(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT));
            sprite.set_num_frames(8, true);
            sprite.set_position(sprite_dying.position().left, sprite_dying.position().top);
            self.engine.add_sprite(sprite);
        }
    }

    //点击、触摸事件
    pub fn on_touch_event(&mut self, event:i32, x:i32, _y:i32){
        match event{
            //点击发射炮弹
            EVENT_MOUSE_CLICK => {
                //如果游戏没有开始，启动游戏
                if self.demo || self.game_over{
                    self.demo = false;
                    self.new_game();
                    return;
                }

                let car_left_pos = self.engine.get_sprite(self.car_sprite_id).unwrap().position().left;
                //创建一个新的导弹精灵
                let mut sprite = Sprite::with_bounds_action(
                    BitmapRes::new(RES_MISSILE_BITMAP, 5, 16),
                    Rect::new(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT), BA_DIE);
                sprite.set_position(car_left_pos+15, 400);
                sprite.set_velocity(0, -7);
                self.engine.add_sprite(sprite);

                //播放导弹发射声音
                GameEngine::play_sound(RES_MISSILE_SOUND);
            },

            EVENT_TOUCH_MOVE | EVENT_MOUSE_MOVE => {
                if self.demo{
                    return;
                }
                let mut car_sprite = self.engine.get_sprite(self.car_sprite_id).unwrap();
                let pos = *car_sprite.position();
                car_sprite.set_position(x, pos.top);
            }

            _ => ()
        }
    }

    pub fn difficulty(&self)->i32{
        self.difficulty
    }

    pub fn engine(&mut self)->&mut GameEngine{
        &mut self.engine
    }
}

//游戏引擎回调函数
struct GameHandler{}
impl GameEngineHandler for GameHandler{
    fn sprite_dying(&mut self, sprite_dying:&Sprite){
        window().game.sprite_dying(sprite_dying);
    }
    fn sprite_collision(&self, sprite_hitter:&Sprite, sprite_hittee:&Sprite)->bool{
        window().game.sprite_collision(sprite_hitter, sprite_hittee)
    }
}