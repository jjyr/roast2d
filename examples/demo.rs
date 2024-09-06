extern crate roast_2d;
use std::cell::{OnceCell, RefCell};

use roast_2d::prelude::*;

const BALL_ACCEL: f32 = 200.0;
const BALL_MAX_VEL: f32 = 300.0;
const PLAYER_VEL: f32 = 600.0;
const FRICTION: f32 = 4.0;
const WALL_THICK: f32 = 200.0;
const BRICK_SIZE: Vec2 = Vec2::new(64., 32.);
const BRICK_DYING: f32 = 0.3;
const TEXTURE_PATH: &str = "demo.png";

thread_local! {
    static G: RefCell<Game> = RefCell::new(Game::default());
    static TEXTURE: OnceCell<Handle> = const { OnceCell::new() } ;
}

fn load_texture(eng: &mut Engine) -> Handle {
    TEXTURE.with(|t| {
        t.get_or_init(|| eng.assets.load_texture(TEXTURE_PATH))
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

impl EntType for Ball {
    fn load(eng: &mut Engine, _w: &mut World) -> Self {
        let size = Vec2::new(32., 32.0);
        let mut sheet = Sprite::with_sizef(load_texture(eng), size);
        sheet.color = Color::rgb(0xfb, 0xf2, 0x36);
        let anim = Animation::new(sheet);
        Ball { size, anim }
    }

    fn init(&mut self, _eng: &mut Engine, w: &mut World, ent: EntRef) {
        let ent = w.get_mut(ent).unwrap();
        ent.size = self.size;
        ent.anim = Some(self.anim.clone());
        ent.group = EntGroup::PROJECTILE;
        ent.accel.y = -BALL_ACCEL * 2.0;
        ent.friction = Vec2::splat(0.1);
        ent.physics = EntPhysics::LITE;
        ent.restitution = 12.0;
    }

    fn collide(
        &mut self,
        _eng: &mut Engine,
        w: &mut World,
        ent: EntRef,
        normal: Vec2,
        _trace: Option<&Trace>,
    ) {
        let ent = w.get_mut(ent).unwrap();

        if normal.y != 0.0 {
            ent.vel.y = normal.y * BALL_MAX_VEL;
            ent.accel.y = normal.y * BALL_ACCEL;
        }
        if normal.x != 0.0 {
            ent.vel.x = normal.x * BALL_MAX_VEL;
            ent.accel.x = normal.x * BALL_ACCEL;
        }
    }

    fn post_update(&mut self, eng: &mut Engine, w: &mut World, ent: EntRef) {
        let ent = w.get_mut(ent).unwrap();
        let view = eng.view_size();
        let half_size = ent.size * 0.5;
        let bounds = ent.bounds();
        if bounds.max.x < 0.0 {
            ent.pos.x = half_size.x;
            ent.vel.x = BALL_MAX_VEL;
        }
        if bounds.min.x > view.x {
            ent.pos.x = view.x - half_size.x;
            ent.vel.x = -BALL_MAX_VEL;
        }
        if bounds.max.y < 0.0 {
            ent.pos.y = half_size.y;
            ent.vel.y = BALL_MAX_VEL;
        }
        if bounds.min.y > view.y {
            ent.pos.y = view.y - half_size.y;
            ent.vel.y = -BALL_MAX_VEL;
        }
    }
}

#[derive(Clone)]
pub struct LeftWall;

impl EntType for LeftWall {
    fn load(_eng: &mut Engine, _w: &mut World) -> Self {
        Self
    }

    fn init(&mut self, eng: &mut Engine, w: &mut World, ent: EntRef) {
        let ent = w.get_mut(ent).unwrap();
        ent.size = Vec2::new(WALL_THICK, eng.view_size().y);
        ent.check_against = EntGroup::PROJECTILE;
        ent.physics = EntPhysics::FIXED;
    }
}

#[derive(Clone)]
pub struct RightWall;

impl EntType for RightWall {
    fn load(_eng: &mut Engine, _w: &mut World) -> Self {
        Self
    }
    fn init(&mut self, eng: &mut Engine, w: &mut World, ent: EntRef) {
        let ent = w.get_mut(ent).unwrap();
        ent.size = Vec2::new(WALL_THICK, eng.view_size().y);
        ent.check_against = EntGroup::PROJECTILE;
        ent.physics = EntPhysics::FIXED;
    }
}

#[derive(Clone)]
pub struct TopWall;

impl EntType for TopWall {
    fn load(_eng: &mut Engine, _w: &mut World) -> Self {
        Self
    }
    fn init(&mut self, eng: &mut Engine, w: &mut World, ent: EntRef) {
        let ent = w.get_mut(ent).unwrap();
        ent.size = Vec2::new(eng.view_size().x, WALL_THICK);
        ent.check_against = EntGroup::PROJECTILE;
        ent.physics = EntPhysics::FIXED;
    }
}

#[derive(Clone)]
pub struct BottomWall;

impl EntType for BottomWall {
    fn load(_eng: &mut Engine, _w: &mut World) -> Self {
        Self
    }
    fn init(&mut self, eng: &mut Engine, w: &mut World, ent: EntRef) {
        let ent = w.get_mut(ent).unwrap();
        ent.size = Vec2::new(eng.view_size().x, WALL_THICK);
        ent.check_against = EntGroup::PROJECTILE;
        ent.physics = EntPhysics::FIXED;
    }
}

#[derive(Clone)]
pub struct Brick {
    hit: bool,
    dying: f32,
    dead_pos: Vec2,
    anim: Animation,
}

impl EntType for Brick {
    fn load(eng: &mut Engine, _w: &mut World) -> Self {
        let mut sheet = Sprite::with_sizef(load_texture(eng), BRICK_SIZE);
        sheet.color = Color::rgb(0x5b, 0x6e, 0xe1);
        let anim = Animation::new(sheet);
        Brick {
            hit: false,
            dying: 0.0,
            dead_pos: Vec2::default(),
            anim,
        }
    }

    fn init(&mut self, _eng: &mut Engine, w: &mut World, ent: EntRef) {
        let ent = w.get_mut(ent).unwrap();
        ent.anim = Some(self.anim.clone());
        ent.size = BRICK_SIZE;
        ent.check_against = EntGroup::PROJECTILE;
        ent.physics = EntPhysics::ACTIVE;
    }

    fn kill(&mut self, _eng: &mut Engine, _w: &mut World, _ent: EntRef) {
        G.with_borrow_mut(|g| {
            g.score += 1;
        });
    }

    fn update(&mut self, eng: &mut Engine, w: &mut World, ent: EntRef) {
        if self.hit {
            self.dying += eng.tick;
            if self.dying > BRICK_DYING {
                eng.kill(ent);
            }

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
            let ent = w.get_mut(ent).unwrap();
            ent.scale = Vec2::splat(scale);
            if let Some(anim) = ent.anim.as_mut() {
                anim.sheet.color = color;
            }
        }
    }

    fn touch(&mut self, _eng: &mut Engine, w: &mut World, ent: EntRef, _other: EntRef) {
        if !self.hit {
            self.hit = true;
            self.dead_pos = w.get(ent).unwrap().pos;
        }
    }
}

#[derive(Clone)]
pub struct Player {
    size: Vec2,
    anim: Animation,
}

impl EntType for Player {
    fn load(eng: &mut Engine, _w: &mut World) -> Self {
        let size = Vec2::new(128.0, 48.0);
        let mut sheet = Sprite::with_sizef(load_texture(eng), size);
        sheet.color = Color::rgb(0x37, 0x94, 0x6e);
        let anim = Animation::new(sheet);

        Self { size, anim }
    }

    fn init(&mut self, _eng: &mut Engine, w: &mut World, ent: EntRef) {
        let ent = w.get_mut(ent).unwrap();
        ent.size = self.size;
        ent.anim = Some(self.anim.clone());
        ent.friction = Vec2::splat(FRICTION);
        ent.check_against = EntGroup::PROJECTILE;
        ent.physics = EntPhysics::ACTIVE;
    }

    fn update(&mut self, eng: &mut Engine, w: &mut World, ent: EntRef) {
        let ent = w.get_mut(ent).unwrap();

        let input = eng.input();

        ent.accel = Vec2::default();
        if input.pressed(Action::Right) {
            ent.vel.x = PLAYER_VEL;
        }
        if input.pressed(Action::Left) {
            ent.vel.x = -PLAYER_VEL;
        }
    }

    fn touch(&mut self, _eng: &mut Engine, w: &mut World, ent: EntRef, other: EntRef) {
        let [Some(ent), Some(other)] = w.get_many_mut([ent, other]) else {
            return;
        };
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
    score_text: Option<Sprite>,
    fps_text: Option<Sprite>,
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
    fn init(&mut self, eng: &mut Engine, w: &mut World) {
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

        eng.spawn::<Player>(w, Vec2::new(108.0, view.y - 8.0));
        eng.spawn::<Ball>(w, Vec2::new(40.0, view.y - 64.0));

        eng.spawn::<LeftWall>(w, Vec2::new(-WALL_THICK * 0.5, view.y * 0.5));
        eng.spawn::<RightWall>(w, Vec2::new(view.x + WALL_THICK * 0.5, view.y * 0.5));
        eng.spawn::<TopWall>(w, Vec2::new(view.x * 0.5, -WALL_THICK * 0.5));
        eng.spawn::<BottomWall>(
            w,
            Vec2::new(eng.view_size().x * 0.5, view.y + WALL_THICK * 0.5),
        );

        let padding = 5.;
        let row_gap = 5.;
        let offset_x = view.x % (BRICK_SIZE.x + padding) * 0.5;
        let cols = (view.x / (BRICK_SIZE.x + padding)) as i32;
        let rows = 6;

        for i in 0..cols {
            for j in 0..rows {
                eng.spawn::<Brick>(
                    w,
                    Vec2::new(
                        (i as f32 + 0.5) * (BRICK_SIZE.x + padding) + offset_x,
                        (j as f32 + 0.5) * (BRICK_SIZE.y + row_gap),
                    ),
                );
            }
        }

        log::info!("Init Demo");
    }

    fn update(&mut self, eng: &mut Engine, w: &mut World) {
        eng.scene_base_update(w);
        self.frames += 1.0;
        self.timer += eng.tick;
        if self.timer > self.interval {
            let fps = self.frames / self.timer;
            self.timer = 0.;
            self.frames = 0.;

            if let Some(font) = self.font.clone() {
                let content = format!("FPS: {:.2}", fps);
                let text = Text::new(content, font, 30.0, WHITE);
                let (texture, size) = eng.create_text_texture(text);
                self.fps_text = Some(Sprite::new(texture, size));
            }
        }
        if let Some(font) = self.font.clone() {
            let score = G.with_borrow(|g| g.score);
            let content = format!("Score: {}", score);
            let text = Text::new(content, font.clone(), 30.0, WHITE);
            let (texture, size) = eng.create_text_texture(text);
            self.score_text = Some(Sprite::new(texture, size));
        }
    }

    fn draw(&mut self, eng: &mut Engine, w: &mut World) {
        eng.scene_base_draw(w);
        if let Some(text) = self.score_text.as_ref() {
            eng.draw_image(text, text.sizef() * 0.5, None, None);
        }
        if let Some(text) = self.fps_text.as_ref() {
            let x = eng.view_size().x - (text.size().x as f32 * 0.5);
            eng.draw_image(text, Vec2::new(x, text.sizef().y * 0.5), None, None);
        }
    }
}

fn setup(eng: &mut Engine, w: &mut World) {
    // set resize and scale
    eng.set_view_size(Vec2::new(800.0, 600.0));
    eng.set_scale_mode(ScaleMode::Exact);
    eng.set_resize_mode(ResizeMode {
        width: true,
        height: true,
    });
    eng.set_sweep_axis(SweepAxis::Y);
    eng.add_ent_type::<Player>(w);
    eng.add_ent_type::<LeftWall>(w);
    eng.add_ent_type::<RightWall>(w);
    eng.add_ent_type::<TopWall>(w);
    eng.add_ent_type::<BottomWall>(w);
    eng.add_ent_type::<Ball>(w);
    eng.add_ent_type::<Brick>(w);
    eng.set_scene(Demo::default());
}

#[cfg(not(target_arch = "wasm32"))]
#[pollster::main]
async fn main() {
    env_logger::init();
    App::default()
        .title("Hello Roast2D".to_string())
        .window(UVec2::new(800, 600))
        .vsync(true)
        .run(setup)
        .await
        .expect("Start game");
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub async fn run() {
    App::default()
        .title("Hello Roast2D".to_string())
        .window(UVec2::new(800, 600))
        .vsync(true)
        .run(setup)
        .await
        .expect("Start game");
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // See run function
}
