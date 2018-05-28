use engine::HtmlImage;
use engine::sprite::{Entity, PointF, Rect, Sprite, BA_DIE, BA_STOP, BA_WRAP};
use engine::utils::rand_int;
use engine::animation::Animation;
use {CLIENT_HEIGHT, CLIENT_WIDTH, PLAYER_LIVES, TANK_BITMAP_HEIGHT, TANK_BITMAP_WIDTH};
use std::cell::RefCell;
use std::rc::Rc;

//--------------------------------------------
//-------------游戏资源ID----------------------
//--------------------------------------------
pub const SPRITE_TANK: i32 = 0;
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
pub const RES_BG_BITMAP: u8 = 6;

pub const SM_GUN_BITMAP: HtmlImage = HtmlImage {
    id: RES_SM_GUN_BITMAP,
    width: 64,
    height: 64,
};

//玩家坦克
pub struct TankSprite {
    entity: Entity,
    pub ip: String,
}

impl TankSprite {
    pub fn new(ip: String, id: u32, position: PointF) -> TankSprite {
        let bitmap = Rc::new(RefCell::new(HtmlImage {
                id: RES_TANK_BITMAP,
                width: TANK_BITMAP_WIDTH as i32 * 4,
                height: TANK_BITMAP_HEIGHT as i32,
            }));
        let mut entity = Entity::new(
            id,
            vec![
                Animation::single_frame(bitmap.clone(), 0, 0, TANK_BITMAP_HEIGHT, TANK_BITMAP_HEIGHT),
                Animation::single_frame(bitmap.clone(), 0, TANK_BITMAP_HEIGHT, TANK_BITMAP_HEIGHT, TANK_BITMAP_HEIGHT),
                Animation::single_frame(bitmap.clone(), 0, TANK_BITMAP_HEIGHT*2, TANK_BITMAP_HEIGHT, TANK_BITMAP_HEIGHT),
                Animation::single_frame(bitmap, 0, TANK_BITMAP_HEIGHT*3, TANK_BITMAP_HEIGHT, TANK_BITMAP_HEIGHT),
            ],
            position,
            Rect::new(0.0, 0.0, CLIENT_WIDTH as f64, CLIENT_HEIGHT as f64),
            BA_STOP,
        );
        entity.lives = PLAYER_LIVES;
        TankSprite { ip, entity }
    }
}

impl Sprite for TankSprite {
    fn class(&self) -> i32 {
        SPRITE_TANK
    }
    fn get_entity(&self) -> &Entity {
        &self.entity
    }
    fn get_entity_mut(&mut self) -> &mut Entity {
        &mut self.entity
    }
}

//护士
pub struct NruseSprite {
    entity: Entity,
    frame_count: u32,
}

impl NruseSprite {
    pub fn new(id: u32, position: Option<PointF>) -> NruseSprite {
        let bitmap = Rc::new(RefCell::new(HtmlImage {
                id: RES_NURSE_BITMAP,
                width: TANK_BITMAP_WIDTH as i32 * 4,
                height: TANK_BITMAP_HEIGHT as i32,
            }));
        let entity = Entity::new(
            id,
            vec![
                Animation::single_frame(bitmap.clone(), 0, 0, TANK_BITMAP_HEIGHT, TANK_BITMAP_HEIGHT),
                Animation::single_frame(bitmap.clone(), 0, TANK_BITMAP_HEIGHT, TANK_BITMAP_HEIGHT, TANK_BITMAP_HEIGHT),
                Animation::single_frame(bitmap.clone(), 0, TANK_BITMAP_HEIGHT*2, TANK_BITMAP_HEIGHT, TANK_BITMAP_HEIGHT),
                Animation::single_frame(bitmap, 0, TANK_BITMAP_HEIGHT*3, TANK_BITMAP_HEIGHT, TANK_BITMAP_HEIGHT),
            ],
            PointF::zero(),
            Rect::new(0.0, 0.0, CLIENT_WIDTH as f64, CLIENT_HEIGHT as f64),
            BA_WRAP,
        );

        let mut nurse = NruseSprite {
            entity,
            frame_count: 0,
        };

        if let Some(position) = position {
            nurse.set_position_point(position.x, position.y);
        } else {
            //随机速度 velocity = 0.05~0.2
            let velocity = rand_int(5, 20) as f64 / 100.0;
            match rand_int(0, 3) {
                1 => {
                    //向下
                    nurse.set_velocity(0.0, velocity);
                    nurse.set_cur_animation(1);
                    nurse.set_position_point(
                        rand_int(TANK_BITMAP_WIDTH as i32, CLIENT_WIDTH - TANK_BITMAP_WIDTH as i32) as f64,
                        -(TANK_BITMAP_HEIGHT as i32) as f64,
                    );
                }
                2 => {
                    //向左
                    nurse.set_velocity(-velocity, 0.0);
                    nurse.set_cur_animation(2);
                    nurse.set_position_point(
                        CLIENT_WIDTH as f64,
                        rand_int(TANK_BITMAP_HEIGHT as i32, CLIENT_HEIGHT - TANK_BITMAP_HEIGHT as i32) as f64,
                    );
                }
                3 => {
                    //向右
                    nurse.set_velocity(velocity, 0.0);
                    nurse.set_cur_animation(3);
                    nurse.set_position_point(
                        0.0,
                        rand_int(TANK_BITMAP_HEIGHT as i32, CLIENT_HEIGHT - TANK_BITMAP_HEIGHT as i32) as f64,
                    );
                    //println!("护士位置: {:?}", nurse.position());
                }
                _ => {
                    //向上
                    nurse.set_velocity(0.0, -velocity);
                    nurse.set_cur_animation(0);
                    nurse.set_position_point(
                        rand_int(TANK_BITMAP_WIDTH as i32, CLIENT_WIDTH - TANK_BITMAP_WIDTH as i32) as f64,
                        CLIENT_HEIGHT as f64,
                    );
                }
            }
        }
        nurse
    }
}

impl Sprite for NruseSprite {
    fn class(&self) -> i32 {
        SPRITE_NURSE
    }
    fn update(&mut self, elapsed_milis: f64) -> u32 {
        self.frame_count += 1;
        //3s之后再设置为 BA_DIE, 防止刚出现就死亡
        if self.frame_count > 60 * 3 {
            self.entity.bounds_action = BA_DIE;
        }
        self.entity.update(elapsed_milis)
    }
    fn get_entity(&self) -> &Entity {
        &self.entity
    }
    fn get_entity_mut(&mut self) -> &mut Entity {
        &mut self.entity
    }
}

//子弹精灵
pub struct MissileSprite {
    entity: Entity,
}

impl MissileSprite {
    pub fn new(id: u32, position: PointF) -> MissileSprite {
        let bitmap = Rc::new(RefCell::new(HtmlImage {
                id: RES_MISSILE_BITMAP,
                width: 80,
                height: 20,
            }));
        let entity = Entity::new(
            id,
            vec![
                Animation::single_frame(bitmap.clone(), 0, 0, 20, 20),
                Animation::single_frame(bitmap.clone(), 0, 20, 20, 20),
                Animation::single_frame(bitmap.clone(), 0, 20*2, 20, 20),
                Animation::single_frame(bitmap.clone(), 0, 20*3, 20, 20),
            ],
            position,
            Rect::new(0.0, 0.0, CLIENT_WIDTH as f64, CLIENT_HEIGHT as f64),
            BA_DIE,
        );
        MissileSprite { entity }
    }
}

impl Sprite for MissileSprite {
    fn class(&self) -> i32 {
        SPRITE_MISSILE
    }
    fn get_entity(&self) -> &Entity {
        &self.entity
    }
    fn get_entity_mut(&mut self) -> &mut Entity {
        &mut self.entity
    }
}

//小爆炸
pub struct SMExplosionSprite {
    entity: Entity,
}

impl SMExplosionSprite {
    pub fn new(id: u32, position: PointF) -> SMExplosionSprite {
        let entity = Entity::new(
            id,
            vec![
                Animation::on_cycle(Rc::new(RefCell::new(HtmlImage {
                    id: RES_SM_EXPLOSION_BITMAP,
                    width: 136,
                    height: 17,
                })), 0, 0, 17, 17, 8, 500)
            ],
            PointF::zero(),
            Rect::new(0.0, 0.0, CLIENT_WIDTH as f64, CLIENT_HEIGHT as f64),
            BA_STOP
        );
        let mut sprite = SMExplosionSprite { entity };
        sprite.set_position_point(position.x, position.y);
        sprite
    }
}

impl Sprite for SMExplosionSprite {
    fn class(&self) -> i32 {
        SPRITE_SM_EXPLOSION
    }
    fn get_entity(&self) -> &Entity {
        &self.entity
    }
    fn get_entity_mut(&mut self) -> &mut Entity {
        &mut self.entity
    }
}

//大爆炸
pub struct LGExplosionSprite {
    entity: Entity,
}

impl LGExplosionSprite {
    pub fn new(id: u32, position: PointF) -> LGExplosionSprite {
        let entity = Entity::new(
            id,
            vec![
                Animation::on_cycle(Rc::new(RefCell::new(HtmlImage {
                id: RES_LG_EXPLOSION_BITMAP,
                width: 272,
                height: 33,
            })), 0, 0, 34, 33, 8, 500)
            ],
            PointF::zero(),
            Rect::new(0.0, 0.0, CLIENT_WIDTH as f64, CLIENT_HEIGHT as f64),
            BA_STOP
        );
        let mut sprite = LGExplosionSprite { entity };
        sprite.set_position_point(position.x, position.y);
        sprite
    }
}

impl Sprite for LGExplosionSprite {
    fn class(&self) -> i32 {
        SPRITE_LG_EXPLOSION
    }
    fn get_entity(&self) -> &Entity {
        &self.entity
    }
    fn get_entity_mut(&mut self) -> &mut Entity {
        &mut self.entity
    }
}
