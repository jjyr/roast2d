use std::cell::RefCell;

use roast2d::{derive::Component, prelude::*};
use roast2d_physics::{
    collision::{init_collision, CollisionSet, SweepAxis},
    entities::{draw_entities, init_commands, update_entities, Commands, EntHooks, Hooks},
    physics::{EntGroup, EntPhysics, Physics},
    trace::Trace,
};

const BALL_ACCEL: f32 = 100.0;
const BALL_MIN_VEL: f32 = 180.0;
const BALL_MAX_VEL: f32 = 280.0;
const PLAYER_VEL: f32 = 400.0;
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
                vel: Vec2::new(0.0, -BALL_MAX_VEL),
                friction: Vec2::splat(0.1),
                physics: EntPhysics::LITE,
                restitution: 12.0,
                gravity: 0.0,
                ..Default::default()
            })
            .add(Ball { size, color })
            .add(Hooks::new(Ball { size, color }))
            .id();

        w.get_resource_mut::<CollisionSet>().unwrap().add(ent);

        ent
    }
}

impl EntHooks for Ball {
    fn draw(&self, g: &mut Engine, w: &mut World, ent: Ent, viewport: Vec2) -> Result<()> {
        let ent = w.get(ent)?;
        let transform = ent.get::<Transform>()?;
        g.draw_rect(
            self.size,
            transform.pos + viewport,
            self.color,
            None,
            Some(transform.scale),
            None,
        );
        Ok(())
    }

    fn collide(
        &self,
        _g: &mut Engine,
        w: &mut World,
        ent: Ent,
        normal: Vec2,
        _trace: Option<&Trace>,
    ) -> Result<()> {
        let mut ent = w.get_mut(ent)?;
        let t = ent.get_mut::<Physics>()?;

        if normal.y != 0.0 {
            t.vel.y = normal.y * BALL_MAX_VEL;
            t.accel.y = normal.y * BALL_ACCEL;
        }
        if normal.x != 0.0 {
            t.vel.x = normal.x * BALL_MAX_VEL;
            t.accel.x = normal.x * BALL_ACCEL;
        }
        Ok(())
    }

    fn post_update(&self, g: &mut Engine, w: &mut World, ent: Ent) -> Result<()> {
        let mut ent = w.get_mut(ent)?;
        let view = g.view_size();
        let t = ent.get::<Transform>()?;
        let half_size = t.size * 0.5;
        let bounds = t.bounds();
        if bounds.max.x < 0.0 {
            ent.get_mut::<Transform>()?.pos.x = half_size.x;
            ent.get_mut::<Physics>()?.vel.x = BALL_MAX_VEL;
        }
        if bounds.min.x > view.x {
            ent.get_mut::<Transform>()?.pos.x = view.x - half_size.x;
            ent.get_mut::<Physics>()?.vel.x = -BALL_MAX_VEL;
        }
        if bounds.max.y < 0.0 {
            ent.get_mut::<Transform>()?.pos.y = half_size.y;
            ent.get_mut::<Physics>()?.vel.y = BALL_MAX_VEL;
        }
        if bounds.min.y > view.y {
            ent.get_mut::<Transform>()?.pos.y = view.y - half_size.y;
            ent.get_mut::<Physics>()?.vel.y = -BALL_MAX_VEL;
        }

        let p = ent.get_mut::<Physics>()?;
        p.vel.y = p.vel.y.abs().clamp(BALL_MIN_VEL, BALL_MAX_VEL) * p.vel.y.signum();

        Ok(())
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
    fn draw(&self, g: &mut Engine, w: &mut World, ent: Ent, viewport: Vec2) -> Result<()> {
        let ent = w.get(ent)?;
        let t = ent.get::<Transform>()?;
        let color = ent.get::<Brick>()?.color;
        g.draw_rect(t.size, t.pos + viewport, color, None, Some(t.scale), None);
        Ok(())
    }

    fn kill(&self, _g: &mut Engine, w: &mut World, ent: Ent) -> Result<()> {
        G.with_borrow_mut(|g| {
            g.score += 1;
        });

        w.get_resource_mut::<CollisionSet>()?.remove(ent);
        Ok(())
    }

    fn update(&self, g: &mut Engine, w: &mut World, ent: Ent) -> Result<()> {
        w.with_resource::<Commands, Result<()>, _>(|w, commands| -> Result<()> {
            let mut ent = w.get_mut(ent)?;
            if ent.get::<Brick>()?.hit {
                let ent_id = ent.id();
                let brick = ent.get_mut::<Brick>()?;
                brick.dying += g.tick;
                if brick.dying > BRICK_DYING {
                    commands.kill(ent_id);
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
                let t = ent.get_mut::<Transform>()?;
                t.scale = Vec2::splat(scale);
            }
            Ok(())
        })
    }

    fn touch(&self, _g: &mut Engine, w: &mut World, ent: Ent, _other: Ent) -> Result<()> {
        let mut ent = w.get_mut(ent)?;
        let brick = ent.get_mut::<Brick>()?;
        if !brick.hit {
            brick.hit = true;
            let pos = ent.get::<Transform>()?.pos;
            ent.get_mut::<Brick>()?.dead_pos = pos;
        }
        Ok(())
    }
}

#[derive(Component)]
pub struct Player {
    color: Color,
}

impl Player {
    pub fn init(w: &mut World, pos: Vec2) -> Ent {
        let size = Vec2::new(160.0, 48.0);
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
    fn draw(&self, g: &mut Engine, w: &mut World, ent: Ent, viewport: Vec2) -> Result<()> {
        let ent = w.get(ent)?;
        let t = ent.get::<Transform>()?;
        let p = ent.get::<Player>()?;
        g.draw_rect(t.size, t.pos + viewport, p.color, None, Some(t.scale), None);
        Ok(())
    }

    fn update(&self, g: &mut Engine, w: &mut World, ent: Ent) -> Result<()> {
        let mut ent = w.get_mut(ent)?;
        let phy = ent.get_mut::<Physics>()?;

        let input = g.input();

        phy.accel = Vec2::default();
        if input.pressed(Action::Right) {
            phy.vel.x = PLAYER_VEL;
        }
        if input.pressed(Action::Left) {
            phy.vel.x = -PLAYER_VEL;
        }
        Ok(())
    }

    fn touch(&self, _g: &mut Engine, w: &mut World, ent: Ent, other: Ent) -> Result<()> {
        let [mut ent, mut other] = w.many_mut([ent, other]);
        if other.get::<Ball>().is_ok() {
            let p1 = ent.get_mut::<Physics>()?;
            let p2 = other.get_mut::<Physics>()?;
            p2.accel.x += p1.vel.x * 0.6;
            p2.vel.x = p2.accel.x.signum() * p2.vel.x.abs();
        }
        Ok(())
    }
}

pub struct Demo {
    frames: f32,
    timer: f32,
    interval: f32,
    fps: f32,
}

impl Default for Demo {
    fn default() -> Self {
        Self {
            frames: 0.0,
            timer: 0.0,
            interval: 1.0,
            fps: 0.0,
        }
    }
}

impl Scene for Demo {
    fn init(&mut self, g: &mut Engine, w: &mut World) {
        let view = g.view_size();

        // enable sub modules
        init_commands(g, w);
        init_collision(g, w, SweepAxis::Y);

        // bind keys
        let input = g.input_mut();
        input.bind(KeyCode::Left, Action::Left);
        input.bind(KeyCode::Right, Action::Right);
        input.bind(KeyCode::KeyA, Action::Left);
        input.bind(KeyCode::KeyD, Action::Right);

        Player::init(w, Vec2::new(108.0, view.y - 8.0));
        Ball::init(w, Vec2::new(40.0, view.y - 64.0));

        // walls
        let v_size = Vec2::new(WALL_THICK, g.view_size().y);
        let h_size = Vec2::new(g.view_size().x, WALL_THICK);
        let l_pos = Vec2::new(-WALL_THICK * 0.5, view.y * 0.5);
        let r_pos = Vec2::new(view.x + WALL_THICK * 0.5, view.y * 0.5);
        let t_pos = Vec2::new(view.x * 0.5, -WALL_THICK * 0.5);
        let b_pos = Vec2::new(g.view_size().x * 0.5, view.y + WALL_THICK * 0.5);
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

    fn update(&mut self, g: &mut Engine, w: &mut World) {
        update_entities(g, w);
        self.frames += 1.0;
        self.timer += g.tick;
        if self.timer > self.interval {
            self.fps = self.frames / self.timer;
            self.timer = 0.;
            self.frames = 0.;
        }
    }

    fn draw(&mut self, g: &mut Engine, w: &mut World) {
        draw_entities(g, w);
        // Score
        let score = G.with_borrow(|g| g.score);
        g.draw_text(
            Text::new(format!("Score: {}", score), 20.0, WHITE),
            Vec2::new(0.0, 0.0),
            Vec2::ZERO,
            None,
        );
        // FPS
        g.draw_text(
            Text::new(format!("FPS: {:.2}", self.fps), 20.0, WHITE),
            Vec2::new(g.view_size().x - 160.0, 0.0),
            Vec2::ZERO,
            None,
        );
    }

    fn cleanup(&mut self, _g: &mut Engine, _w: &mut World) {}
}

fn setup(g: &mut Engine, _w: &mut World) {
    // set resize and scale
    g.set_view_size(Vec2::new(800.0, 600.0));
    g.set_scale_mode(ScaleMode::Exact);
    g.set_resize_mode(ResizeMode {
        width: true,
        height: true,
    });
    g.set_scene(Demo::default());
}

async fn run() {
    App::default()
        .title("Hello Roast2D".to_string())
        .window(UVec2::new(800, 600))
        .vsync(true)
        .run(setup)
        .await
        .expect("Start game");
}

#[cfg(not(target_arch = "wasm32"))]
#[pollster::main]
async fn main() {
    env_logger::init();
    run().await;
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // See run function
}
