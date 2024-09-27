extern crate roast2d;
use std::cell::RefCell;

use roast2d::{collision::CollisionSet, hooks::Hooks, prelude::*};
use roast2d_derive::Component;

const BALL_ACCEL: f32 = 200.0;
const BALL_MAX_VEL: f32 = 300.0;
const PLAYER_VEL: f32 = 600.0;
const FRICTION: f32 = 4.0;
const WALL_THICK: f32 = 200.0;
const BRICK_SIZE: Vec2 = Vec2::new(64., 32.);
const BRICK_DYING: f32 = 0.3;

thread_local! {
    static G: RefCell<Game> = RefCell::new(Game::default());
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

#[derive(Component)]
pub struct Ball {
    size: Vec2,
    color: Color,
}

impl Ball {
    pub fn init(w: &mut World, pos: Vec2) -> Ent {
        let size = Vec2::new(32., 32.0);
        let color = Color::rgb(0xfb, 0xf2, 0x36);

        let ent = w
            .spawn()
            .add(Transform::new(pos, size))
            .add(Physics {
                group: EntGroup::PROJECTILE,
                accel: Vec2::new(0.0, -BALL_ACCEL * 2.0),
                friction: Vec2::splat(0.1),
                physics: EntPhysics::LITE,
                restitution: 12.0,
                ..Default::default()
            })
            .add(Hooks::new(Ball { size, color }))
            .id();

        w.get_resource_mut::<CollisionSet>().unwrap().add(ent);

        ent
    }
}

impl EntHooks for Ball {
    fn draw(&self, eng: &mut Engine, w: &mut World, ent: Ent, viewport: Vec2) {
        let ent = w.get(ent).unwrap();
        if let Some(transform) = ent.get::<Transform>() {
            eng.draw_rect(
                self.size,
                transform.pos + viewport,
                Some(self.color),
                Some(transform.scale),
                None,
            );
        }
    }

    fn collide(
        &self,
        _eng: &mut Engine,
        w: &mut World,
        ent: Ent,
        normal: Vec2,
        _trace: Option<&Trace>,
    ) {
        let mut ent = w.get_mut(ent).unwrap();
        let t = ent.get_mut::<Physics>().unwrap();

        if normal.y != 0.0 {
            t.vel.y = normal.y * BALL_MAX_VEL;
            t.accel.y = normal.y * BALL_ACCEL;
        }
        if normal.x != 0.0 {
            t.vel.x = normal.x * BALL_MAX_VEL;
            t.accel.x = normal.x * BALL_ACCEL;
        }
    }

    fn post_update(&self, eng: &mut Engine, w: &mut World, ent: Ent) {
        let mut ent = w.get_mut(ent).unwrap();
        let view = eng.view_size();
        let t = ent.get::<Transform>().unwrap();
        let half_size = t.size * 0.5;
        let bounds = t.bounds();
        if bounds.max.x < 0.0 {
            if let Some(t) = ent.get_mut::<Transform>() {
                t.pos.x = half_size.x;
            }
            if let Some(p) = ent.get_mut::<Physics>() {
                p.vel.x = BALL_MAX_VEL;
            }
        }
        if bounds.min.x > view.x {
            if let Some(t) = ent.get_mut::<Transform>() {
                t.pos.x = view.x - half_size.x;
            }
            if let Some(p) = ent.get_mut::<Physics>() {
                p.vel.x = -BALL_MAX_VEL;
            }
        }
        if bounds.max.y < 0.0 {
            if let Some(t) = ent.get_mut::<Transform>() {
                t.pos.y = half_size.y;
            }
            if let Some(p) = ent.get_mut::<Physics>() {
                p.vel.y = BALL_MAX_VEL;
            }
        }
        if bounds.min.y > view.y {
            if let Some(t) = ent.get_mut::<Transform>() {
                t.pos.y = view.y - half_size.y;
            }
            if let Some(p) = ent.get_mut::<Physics>() {
                p.vel.y = -BALL_MAX_VEL;
            }
        }
    }
}

#[derive(Default, Component)]
pub struct Wall;

impl Wall {
    fn init(w: &mut World, pos: Vec2, size: Vec2) -> Ent {
        let ent = w
            .spawn()
            .add(Transform::new(pos, size))
            .add(Physics {
                check_against: EntGroup::PROJECTILE,
                physics: EntPhysics::FIXED,
                ..Default::default()
            })
            .add(Wall)
            .id();
        w.get_resource_mut::<CollisionSet>().unwrap().add(ent);
        ent
    }
}

#[derive(Component)]
pub struct Brick {
    hit: bool,
    dying: f32,
    dead_pos: Vec2,
    color: Color,
}

impl Brick {
    pub fn init(w: &mut World, pos: Vec2) -> Ent {
        let color = Color::rgb(0x5b, 0x6e, 0xe1);
        let ent = w
            .spawn()
            .add(Transform::new(pos, BRICK_SIZE))
            .add(Physics {
                check_against: EntGroup::PROJECTILE,
                physics: EntPhysics::ACTIVE,
                ..Default::default()
            })
            .add(Brick {
                hit: false,
                dying: 0.0,
                dead_pos: Vec2::default(),
                color,
            })
            .add(Hooks::new(BrickHooks))
            .id();
        w.get_resource_mut::<CollisionSet>().unwrap().add(ent);
        ent
    }
}

#[derive(Default)]
pub struct BrickHooks;

impl EntHooks for BrickHooks {
    fn draw(&self, eng: &mut Engine, w: &mut World, ent: Ent, viewport: Vec2) {
        let ent = w.get(ent).unwrap();
        let t = ent.get::<Transform>().unwrap();
        let color = ent.get::<Brick>().unwrap().color;
        eng.draw_rect(t.size, t.pos + viewport, Some(color), Some(t.scale), None);
    }

    fn kill(&self, _eng: &mut Engine, w: &mut World, ent: Ent) {
        G.with_borrow_mut(|g| {
            g.score += 1;
        });

        w.get_resource_mut::<CollisionSet>().unwrap().remove(ent);
    }

    fn update(&self, eng: &mut Engine, w: &mut World, ent: Ent) {
        let mut ent = w.get_mut(ent).unwrap();
        if ent.get::<Brick>().unwrap().hit {
            let ent_id = ent.id();
            let brick = ent.get_mut::<Brick>().unwrap();
            brick.dying += eng.tick;
            if brick.dying > BRICK_DYING {
                eng.kill(ent_id);
            }

            let progress = (brick.dying / BRICK_DYING).powi(2);
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
            brick.color = color;
            let t = ent.get_mut::<Transform>().unwrap();
            t.scale = Vec2::splat(scale);
        }
    }

    fn touch(&self, _eng: &mut Engine, w: &mut World, ent: Ent, _other: Ent) {
        let mut ent = w.get_mut(ent).unwrap();
        let brick = ent.get_mut::<Brick>().unwrap();
        if !brick.hit {
            brick.hit = true;
            let pos = ent.get::<Transform>().unwrap().pos;
            ent.get_mut::<Brick>().unwrap().dead_pos = pos;
        }
    }
}

#[derive(Component)]
pub struct Player {
    color: Color,
}

impl Player {
    pub fn init(w: &mut World, pos: Vec2) -> Ent {
        let size = Vec2::new(128.0, 48.0);
        let color = Color::rgb(0x37, 0x94, 0x6e);
        let ent = w
            .spawn()
            .add(Transform::new(pos, size))
            .add(Physics {
                friction: Vec2::splat(FRICTION),
                check_against: EntGroup::PROJECTILE,
                physics: EntPhysics::ACTIVE,
                ..Default::default()
            })
            .add(Player { color })
            .add(Hooks::new(PlayerHooks))
            .id();
        w.get_resource_mut::<CollisionSet>().unwrap().add(ent);
        ent
    }
}

#[derive(Default)]
pub struct PlayerHooks;

impl EntHooks for PlayerHooks {
    fn draw(&self, eng: &mut Engine, w: &mut World, ent: Ent, viewport: Vec2) {
        let ent = w.get(ent).unwrap();
        let t = ent.get::<Transform>().unwrap();
        let p = ent.get::<Player>().unwrap();
        eng.draw_rect(t.size, t.pos + viewport, Some(p.color), Some(t.scale), None);
    }

    fn update(&self, eng: &mut Engine, w: &mut World, ent: Ent) {
        let mut ent = w.get_mut(ent).unwrap();
        let phy = ent.get_mut::<Physics>().unwrap();

        let input = eng.input();

        phy.accel = Vec2::default();
        if input.pressed(Action::Right) {
            phy.vel.x = PLAYER_VEL;
        }
        if input.pressed(Action::Left) {
            phy.vel.x = -PLAYER_VEL;
        }
    }

    fn touch(&self, _eng: &mut Engine, w: &mut World, ent: Ent, other: Ent) {
        let [Some(mut ent), Some(mut other)] = w.get_many_mut([ent, other]) else {
            return;
        };
        if other.get::<Ball>().is_some() {
            let p1 = ent.get_mut::<Physics>().unwrap();
            let p2 = other.get_mut::<Physics>().unwrap();
            p2.vel.x = (p2.vel.x * 0.5 + p1.vel.x).clamp(-BALL_MAX_VEL, BALL_MAX_VEL);
            p2.accel.x = p2.vel.normalize().x * p2.accel.x.abs();
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

        Player::init(w, Vec2::new(108.0, view.y - 8.0));
        Ball::init(w, Vec2::new(40.0, view.y - 64.0));

        // walls
        let v_size = Vec2::new(WALL_THICK, eng.view_size().y);
        let h_size = Vec2::new(eng.view_size().x, WALL_THICK);
        let l_pos = Vec2::new(-WALL_THICK * 0.5, view.y * 0.5);
        let r_pos = Vec2::new(view.x + WALL_THICK * 0.5, view.y * 0.5);
        let t_pos = Vec2::new(view.x * 0.5, -WALL_THICK * 0.5);
        let b_pos = Vec2::new(eng.view_size().x * 0.5, view.y + WALL_THICK * 0.5);
        Wall::init(w, l_pos, v_size);
        Wall::init(w, r_pos, v_size);
        Wall::init(w, t_pos, h_size);
        Wall::init(w, b_pos, h_size);

        let padding = 5.;
        let row_gap = 5.;
        let offset_x = view.x % (BRICK_SIZE.x + padding) * 0.5;
        let cols = (view.x / (BRICK_SIZE.x + padding)) as i32;
        let rows = 6;

        for i in 0..cols {
            for j in 0..rows {
                Brick::init(
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
    w.init_component::<Player>();
    w.init_component::<Wall>();
    w.init_component::<Ball>();
    w.init_component::<Brick>();
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
