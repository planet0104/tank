use engine::sprite::{Entity, Rect, PointF, Sprite, BA_DIE, BA_WRAP};
use engine::{HtmlImage};
use {PLAYER_LIVES, CLIENT_WIDTH, CLIENT_HEIGHT, TANK_BITMAP_WIDTH, TANK_BITMAP_HEIGHT};
use engine::utils::rand_int;

//--------------------------------------------
//-------------游戏资源ID----------------------
//--------------------------------------------
pub const SPRITE_TANK:i32 = 0;
pub const SPRITE_MISSILE: i32 = 1;
pub const SPRITE_LG_EXPLOSION: i32 = 2;
pub const SPRITE_SM_EXPLOSION: i32 = 3;
pub const SPRITE_SM_GUN: i32 = 4;
pub const SPRITE_NURSE: i32 = 5;

//--------------------------------------------
//-------------游戏资源ID----------------------
//--------------------------------------------
pub const RES_TANK_BITMAP: u8 = 0;
pub const RES_MISSILE_BITMAP: u8 = 1;
pub const RES_LG_EXPLOSION_BITMAP: u8 = 2;
pub const RES_SM_EXPLOSION_BITMAP: u8 = 3;
pub const RES_SM_GUN_BITMAP: u8 = 4;
pub const RES_NURSE_BITMAP: u8 = 5;

// pub const TANK_BITMAP: HtmlImage = ;
// pub const MISSILE_BITMAP: HtmlImage = ;
// pub const LG_EXPLOSION_BITMAP: HtmlImage = ;
// pub const SM_EXPLOSION_BITMAP: HtmlImage = ;
// pub const SM_GUN_BITMAP: HtmlImage = HtmlImage{ id:RES_SM_GUN_BITMAP, width: 64, height: 64};

//玩家坦克
pub struct TankSprite{
    entity: Entity,
    pub ip: String
}

impl TankSprite{
    pub fn new(ip:String, id: u32, position: PointF) -> TankSprite{
        let entity = Entity::with_bounds_action(
                    id,
                    Box::new(HtmlImage{ id:RES_TANK_BITMAP, width: TANK_BITMAP_WIDTH, height:TANK_BITMAP_HEIGHT * 4}),
                    position,
                    Rect::new(0.0, 0.0, CLIENT_WIDTH as f64, CLIENT_HEIGHT as f64),
                    BA_WRAP,
                );
        entity.set_num_frames(4, false);
        entity.set_frame_delay(-1);
        entity.lives = PLAYER_LIVES;
        TankSprite{ ip, entity }
    }
}

impl Sprite for TankSprite{
    fn class(&self) -> i32{
        SPRITE_TANK
    }
}

//护士
pub struct NruseSprite{
    entity: Entity,
}

impl NruseSprite{
    pub fn new(id: u32, position: Option<PointF>) -> NruseSprite{
        let entity = Entity::with_bounds_action(
                    id,
                    Box::new(HtmlImage{ id:RES_NURSE_BITMAP, width: TANK_BITMAP_WIDTH, height:TANK_BITMAP_HEIGHT * 4}),
                    PointF::zero(),
                    Rect::new(0.0, 0.0, CLIENT_WIDTH as f64, CLIENT_HEIGHT as f64),
                    BA_WRAP,
                );
        entity.set_num_frames(4, false);
        entity.set_frame_delay(-1);

        let mut nurse = NruseSprite{ entity };

        if let Some(position) = position{
            nurse.set_position_point(position.x, position.y);
        }else{
            //随机速度 velocity = 0.05~0.2
            let velocity = rand_int(5, 20) as f64 / 100.0;
            match rand_int(0, 3) {
                1 => {
                    //向下
                    nurse.set_velocity(0.0, velocity);
                    nurse.set_cur_frame(1);
                    nurse.set_position_point(
                        rand_int(TANK_BITMAP_WIDTH, CLIENT_WIDTH - TANK_BITMAP_WIDTH) as f64,
                        -TANK_BITMAP_HEIGHT as f64,
                    );
                }
                2 => {
                    //向左
                    nurse.set_velocity(-velocity, 0.0);
                    nurse.set_cur_frame(2);
                    nurse.set_position_point(
                        CLIENT_WIDTH as f64,
                        rand_int(TANK_BITMAP_HEIGHT, CLIENT_HEIGHT - TANK_BITMAP_HEIGHT) as f64,
                    );
                }
                3 => {
                    //向右
                    nurse.set_velocity(velocity, -TANK_BITMAP_WIDTH as f64);
                    nurse.set_cur_frame(3);
                    nurse.set_position_point(
                        0.0,
                        rand_int(TANK_BITMAP_HEIGHT, CLIENT_HEIGHT - TANK_BITMAP_HEIGHT) as f64,
                    );
                }
                _ => {
                    //向上
                    nurse.set_velocity(0.0, -velocity);
                    nurse.set_cur_frame(0);
                    nurse.set_position_point(
                        rand_int(TANK_BITMAP_WIDTH, CLIENT_WIDTH - TANK_BITMAP_WIDTH) as f64,
                        CLIENT_HEIGHT as f64,
                    );
                }
            }
        }
        nurse
    }
}

impl Sprite for NruseSprite{
    fn class(&self) -> i32{
        SPRITE_NURSE
    }
}

//子弹精灵
pub struct MissileSprite{
    entity: Entity,
}

impl MissileSprite{
    pub fn new(id: u32, position: PointF) -> MissileSprite{
        let entity = Entity::with_bounds_action(
                    id,
                    Box::new(HtmlImage{ id:RES_MISSILE_BITMAP, width: 20, height: 80}),
                    position,
                    Rect::new(0.0, 0.0, CLIENT_WIDTH as f64, CLIENT_HEIGHT as f64),
                    BA_DIE,
                );
        entity.set_num_frames(4, false);
        entity.set_frame_delay(-1);
        MissileSprite{ entity }
    }
}

impl Sprite for MissileSprite{
    fn class(&self) -> i32{
        SPRITE_MISSILE
    }
}

//小爆炸
pub struct SMExplosionSprite{
    entity: Entity,
}

impl SMExplosionSprite{
    pub fn new(id: u32, position: PointF) -> SMExplosionSprite{
        let entity = Entity::from_bitmap(
                    id,
                    Box::new(HtmlImage{ id:RES_SM_EXPLOSION_BITMAP, width: 17, height: 136}),
                    Rect::new(0.0, 0.0, CLIENT_WIDTH as f64, CLIENT_HEIGHT as f64),
                );
        entity.set_num_frames(8, true);
        entity.set_frame_delay(-1);
        let sprite = SMExplosionSprite{ entity };
        sprite.set_position_point(position.x, position.y);
        sprite
    }
}

impl Sprite for SMExplosionSprite{
    fn class(&self) -> i32{
        SPRITE_SM_EXPLOSION
    }
}

//大爆炸
pub struct LGExplosionSprite{
    entity: Entity,
}

impl LGExplosionSprite{
    pub fn new(id: u32, position: PointF) -> LGExplosionSprite{
        let entity = Entity::from_bitmap(
                    id,
                    Box::new(HtmlImage{ id:RES_LG_EXPLOSION_BITMAP, width: 33, height: 272}),
                    Rect::new(0.0, 0.0, CLIENT_WIDTH as f64, CLIENT_HEIGHT as f64),
                );
        entity.set_num_frames(8, true);
        entity.set_frame_delay(-1);
        let sprite = LGExplosionSprite{ entity };
        sprite.set_position_point(position.x, position.y);
        sprite
    }
}

impl Sprite for LGExplosionSprite{
    fn class(&self) -> i32{
        SPRITE_LG_EXPLOSION
    }
}