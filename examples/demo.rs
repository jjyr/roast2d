extern crate roast_2d;
use std::cell::{OnceCell, RefCell};

use glam::UVec2;
use roast_2d::prelude::*;

const BALL_ACCEL: f32 = 200.0;
const BALL_MAX_VEL: f32 = 300.0;
const PLAYER_VEL: f32 = 600.0;
const FRICTION: f32 = 4.0;
const WALL_THICK: f32 = 200.0;
const SPRITE_SIZE: f32 = 8.0;
const BRICK_SIZE: Vec2 = Vec2::new(64., 32.);
const BRICK_DYING: f32 = 0.3;
const TEXTURE_PATH: &str = "demo.png";

thread_local! {
    static G: RefCell<Game> = RefCell::new(Game::default());
    static TEXTURE: OnceCell<Image> = const { OnceCell::new() } ;
}

fn load_texture(eng: &mut Engine) -> Image {
    TEXTURE.with(|t| {
        t.get_or_init(|| eng.load_image(TEXTURE_PATH).unwrap())
            .clone()
    })
}

#[derive(Default)]
pub struct Game {
    pub score: usize,
}

#[repr(u8)]
pub enum Action {
    Left = 1,
    Right,
    Up,
    Down,
}

impl From<Action> for ActionId {
    fn from(value: Action) -> Self {
        ActionId(value as u8)
    }
}

#[derive(Clone)]
pub struct Ball {
    size: Vec2,
    anim: Animation,
}

impl EntityType for Ball {
    fn load(eng: &mut Engine) -> Self {
        let size = Vec2::new(32., 32.0);
        let mut sheet = load_texture(eng);
        sheet.scale = size / SPRITE_SIZE;
        sheet.color = Color::rgb(0xfb, 0xf2, 0x36);
        let anim = Animation::new(sheet);
        Ball { size, anim }
    }

    fn init(&mut self, _eng: &mut Engine, ent: &mut Entity) {
        ent.size = self.size;
        ent.anim = Some(self.anim.clone());
        ent.group = EntityGroup::PROJECTILE;
        ent.accel.y = -BALL_ACCEL * 2.0;
        ent.friction = Vec2::splat(0.1);
        ent.physics = EntityPhysics::LITE;
        ent.restitution = 12.0;
    }

    fn collide(
        &mut self,
        _eng: &mut Engine,
        ent: &mut Entity,
        normal: Vec2,
        _trace: Option<&Trace>,
    ) {
        if normal.y != 0.0 {
            ent.vel.y = normal.y * BALL_MAX_VEL;
            ent.accel.y = normal.y * BALL_ACCEL;
        }
        if normal.x != 0.0 {
            ent.vel.x = normal.x * BALL_MAX_VEL;
            ent.accel.x = normal.x * BALL_ACCEL;
        }
    }

    fn post_update(&mut self, eng: &mut Engine, ent: &mut Entity) {
        let view = eng.view_size();
        if ent.pos.x < -ent.size.x {
            ent.pos.x = 0.0;
            ent.vel.x = BALL_MAX_VEL;
        }
        if ent.pos.x > view.x {
            ent.pos.x = view.x - ent.size.x;
            ent.vel.x = -BALL_MAX_VEL;
        }
        if ent.pos.y < -ent.size.x {
            ent.pos.y = 0.0;
            ent.vel.y = BALL_MAX_VEL;
        }
        if ent.pos.y > view.y {
            ent.pos.y = view.y - ent.size.y;
            ent.vel.y = -BALL_MAX_VEL;
        }
    }
}

#[derive(Clone)]
pub struct LeftWall;

impl EntityType for LeftWall {
    fn load(_eng: &mut Engine) -> Self {
        Self
    }
    fn init(&mut self, eng: &mut Engine, ent: &mut Entity) {
        ent.size = Vec2::new(WALL_THICK, eng.view_size().y);
        ent.check_against = EntityGroup::PROJECTILE;
        ent.physics = EntityPhysics::FIXED;
    }
}

#[derive(Clone)]
pub struct RightWall;

impl EntityType for RightWall {
    fn load(_eng: &mut Engine) -> Self {
        Self
    }
    fn init(&mut self, eng: &mut Engine, ent: &mut Entity) {
        ent.size = Vec2::new(WALL_THICK, eng.view_size().y);
        ent.check_against = EntityGroup::PROJECTILE;
        ent.physics = EntityPhysics::FIXED;
    }
}

#[derive(Clone)]
pub struct TopWall;

impl EntityType for TopWall {
    fn load(_eng: &mut Engine) -> Self {
        Self
    }
    fn init(&mut self, eng: &mut Engine, ent: &mut Entity) {
        ent.size = Vec2::new(eng.view_size().x, WALL_THICK);
        ent.check_against = EntityGroup::PROJECTILE;
        ent.physics = EntityPhysics::FIXED;
    }
}

#[derive(Clone)]
pub struct BottomWall;

impl EntityType for BottomWall {
    fn load(_eng: &mut Engine) -> Self {
        Self
    }
    fn init(&mut self, eng: &mut Engine, ent: &mut Entity) {
        ent.size = Vec2::new(eng.view_size().x, WALL_THICK);
        ent.check_against = EntityGroup::PROJECTILE;
        ent.physics = EntityPhysics::FIXED;
    }
}

#[derive(Clone)]
pub struct Brick {
    hit: bool,
    dying: f32,
    dead_pos: Vec2,
    anim: Animation,
}

impl EntityType for Brick {
    fn load(eng: &mut Engine) -> Self {
        let mut sheet = load_texture(eng);
        sheet.scale = BRICK_SIZE / SPRITE_SIZE;
        sheet.color = Color::rgb(0x5b, 0x6e, 0xe1);
        let anim = Animation::new(sheet);
        Brick {
            hit: false,
            dying: 0.0,
            dead_pos: Vec2::default(),
            anim,
        }
    }

    fn init(&mut self, _eng: &mut Engine, ent: &mut Entity) {
        ent.anim = Some(self.anim.clone());
        ent.size = BRICK_SIZE;
        ent.check_against = EntityGroup::PROJECTILE;
        ent.physics = EntityPhysics::ACTIVE;
    }

    fn kill(&mut self, _eng: &mut Engine, _ent: &mut Entity) {
        G.with_borrow_mut(|g| {
            g.score += 1;
        });
    }

    fn update(&mut self, eng: &mut Engine, ent: &mut Entity) {
        if self.hit {
            self.dying += eng.tick;
            if self.dying > BRICK_DYING {
                eng.kill(ent.ent_ref);
            }

            if let Some(anim) = ent.anim.as_mut() {
                let progress = (self.dying / BRICK_DYING).powi(2);
                let color = {
                    let (r1, g1, b1): (u8, u8, u8) = (0x5b, 0x6e, 0xe1);
                    let (r2, g2, b2) = (RED.r, RED.g, RED.b);
                    let r = r1.saturating_add(((r1 as f32 - r2 as f32) * progress).abs() as u8);
                    let g = g1.saturating_add(((g1 as f32 - g2 as f32) * progress).abs() as u8);
                    let b = b1.saturating_add(((b1 as f32 - b2 as f32) * progress).abs() as u8);
                    Color::rgb(r, g, b)
                };
                let scale = {
                    let start = 1.0;
                    let end = start * 0.5;
                    start - (start - end) * progress
                };
                let size = BRICK_SIZE * scale;
                let center_pos = self.dead_pos + BRICK_SIZE * 0.5;
                ent.pos = center_pos - size * 0.5;
                ent.size = size;

                anim.sheet.scale = size / SPRITE_SIZE;
                anim.sheet.color = color;
            }
        }
    }

    fn touch(&mut self, _eng: &mut Engine, ent: &mut Entity, _other: &mut Entity) {
        if !self.hit {
            self.hit = true;
            self.dead_pos = ent.pos;
        }
    }
}

#[derive(Clone)]
pub struct Player {
    size: Vec2,
    anim: Animation,
}

impl EntityType for Player {
    fn load(eng: &mut Engine) -> Self {
        let mut sheet = load_texture(eng);
        let size = Vec2::new(128.0, 48.0);
        sheet.scale = size / SPRITE_SIZE;
        sheet.color = Color::rgb(0x37, 0x94, 0x6e);
        let anim = Animation::new(sheet);

        Self { size, anim }
    }

    fn init(&mut self, _eng: &mut Engine, ent: &mut Entity) {
        ent.size = self.size;
        ent.anim = Some(self.anim.clone());
        ent.friction = Vec2::splat(FRICTION);
        ent.check_against = EntityGroup::PROJECTILE;
        ent.physics = EntityPhysics::ACTIVE;
    }

    fn update(&mut self, eng: &mut Engine, ent: &mut Entity) {
        let input = eng.input();

        ent.accel = Vec2::default();
        if input.pressed(Action::Right) {
            ent.vel.x = PLAYER_VEL;
        }
        if input.pressed(Action::Left) {
            ent.vel.x = -PLAYER_VEL;
        }
    }

    fn touch(&mut self, _eng: &mut Engine, ent: &mut Entity, other: &mut Entity) {
        if other.ent_type.is::<Ball>() {
            other.vel.x = (other.vel.x * 0.5 + ent.vel.x).clamp(-BALL_MAX_VEL, BALL_MAX_VEL);
            other.accel.x = other.vel.normalize().x * other.accel.x.abs();
        }
    }
}

pub struct Demo {
    frames: f32,
    timer: f32,
    interval: f32,
    font: Option<Font>,
    score_text: Option<Image>,
    fps_text: Option<Image>,
}

impl Default for Demo {
    fn default() -> Self {
        Self {
            frames: 0.0,
            timer: 0.0,
            interval: 1.0,
            score_text: None,
            font: None,
            fps_text: None,
        }
    }
}

impl Scene for Demo {
    fn init(&mut self, eng: &mut Engine) {
        let view = eng.view_size();

        // bind keys
        let input = eng.input_mut();
        input.bind(KeyCode::Left, Action::Left);
        input.bind(KeyCode::Right, Action::Right);
        input.bind(KeyCode::KeyA, Action::Left);
        input.bind(KeyCode::KeyD, Action::Right);

        // TODO the font path only works on MacOS
        let font_path = "/Library/Fonts/Arial Unicode.ttf";
        if let Ok(font) = Font::open(font_path) {
            self.font.replace(font);
        } else {
            log::error!("Failed to load font from {font_path}");
        }

        eng.spawn::<Player>(Vec2::new(40.0, view.y - 32.0));
        eng.spawn::<Ball>(Vec2::new(40.0, view.y - 64.0));

        eng.spawn::<LeftWall>(Vec2::new(-WALL_THICK, 0.0));
        eng.spawn::<RightWall>(Vec2::new(view.x, 0.0));
        eng.spawn::<TopWall>(Vec2::new(0.0, -WALL_THICK));
        eng.spawn::<BottomWall>(Vec2::new(0.0, view.y));

        let padding = 5.;
        let row_gap = 5.;
        let offset_x = view.x % (BRICK_SIZE.x + padding) * 0.5;
        let cols = (view.x / (BRICK_SIZE.x + padding)) as i32;
        let rows = 6;

        for i in 0..cols {
            for j in 0..rows {
                eng.spawn::<Brick>(Vec2::new(
                    i as f32 * (BRICK_SIZE.x + padding) + offset_x,
                    j as f32 * (BRICK_SIZE.y + row_gap),
                ));
            }
        }

        log::error!("Init Demo");
    }

    fn update(&mut self, eng: &mut Engine) {
        eng.scene_base_update();
        self.frames += 1.0;
        self.timer += eng.tick;
        if self.timer > self.interval {
            let fps = self.frames / self.timer;
            self.timer = 0.;
            self.frames = 0.;

            if let Some(font) = self.font.clone() {
                let content = format!("FPS: {:.2}", fps);
                let text = Text::new(content, font, 30.0, WHITE);
                self.fps_text = eng.create_text_texture(text).ok();
            }
        }
        if let Some(font) = self.font.clone() {
            let score = G.with_borrow(|g| g.score);
            let content = format!("Score: {}", score);
            let text = Text::new(content, font.clone(), 30.0, WHITE);
            self.score_text = eng.create_text_texture(text).ok();
        }
    }

    fn draw(&mut self, eng: &mut Engine) {
        eng.scene_base_draw();
        if let Some(text) = self.score_text.as_ref() {
            eng.draw_image(text, Vec2::new(0.0, 0.0));
        }
        if let Some(text) = self.fps_text.as_ref() {
            let x = eng.view_size().x - text.size().x as f32;
            eng.draw_image(text, Vec2::new(x, 0.0));
        }
    }
}

fn main() {
    App::default()
        .title("Hello Roast2D".to_string())
        .window(UVec2::new(800, 600))
        .vsync(true)
        .run(|eng: &mut Engine| {
            // set resize and scale
            eng.set_view_size(Vec2::new(800.0, 600.0));
            eng.set_scale_mode(ScaleMode::Exact);
            eng.set_resize_mode(ResizeMode {
                width: true,
                height: true,
            });
            eng.set_sweep_axis(SweepAxis::Y);
            eng.add_entity_type::<Player>();
            eng.add_entity_type::<LeftWall>();
            eng.add_entity_type::<RightWall>();
            eng.add_entity_type::<TopWall>();
            eng.add_entity_type::<BottomWall>();
            eng.add_entity_type::<Ball>();
            eng.add_entity_type::<Brick>();
            eng.set_scene(Demo::default());
        })
        .expect("Start game");
}
