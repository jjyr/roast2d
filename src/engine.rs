use std::{
    any::Any,
    cell::{RefCell, UnsafeCell},
    path::Path,
    sync::OnceLock,
};

use anyhow::{bail, Result};
use glam::{UVec2, Vec2};

use crate::{
    asset::{Asset, AssetManager, AssetType, FetchedTask},
    camera::Camera,
    collision::{self, init_collision},
    collision_map::{CollisionMap, COLLISION_MAP},
    color::Color,
    commands::{Command, Commands},
    ecs::{entity::Ent, entity_ref::EntMut, world::World},
    font::{Font, Text},
    handle::Handle,
    health::Health,
    hooks::get_ent_hooks,
    input::InputState,
    ldtk::{LayerType, LdtkProject},
    map::{map_draw, Map},
    physics::{self, entity_move},
    platform::Platform,
    render::{Render, ResizeMode, ScaleMode},
    sprite::Sprite,
    text_cache::{init_text_cache, TextCache},
    trace::Trace,
    transform::Transform,
    types::SweepAxis,
};

/// Default texture
static DEFAULT_TEXTURE: OnceLock<Handle> = OnceLock::new();
/// Max tick
const ENGINE_MAX_TICK: f32 = 100.0;
/// Default font
const DEFAULT_FONT_BYTES: &[u8; 59164] = include_bytes!("../assets/Pixel Square 10.ttf");

/// get default texture
fn default_texture(eng: &mut Engine) -> Handle {
    DEFAULT_TEXTURE
        .get_or_init(|| {
            let handle = eng.assets.alloc_handle();
            let data = vec![255, 255, 255, 255];
            let size = UVec2::splat(1);
            eng.with_platform(|p| {
                p.create_texture(handle.clone(), data, size);
            });
            handle
        })
        .clone()
}

// Scene trait
pub trait Scene {
    // Init the scene, use it to load assets and setup entities.
    fn init(&mut self, _eng: &mut Engine, _w: &mut World) {}

    // Update scene per frame, you probably want to call scene_base_update if you override this function.
    fn update(&mut self, eng: &mut Engine, w: &mut World) {
        eng.scene_base_update(w);
    }

    // Draw scene per frame, use it to draw entities or Hud, you probably want to call scene_base_draw if you override this function.
    fn draw(&mut self, eng: &mut Engine, w: &mut World) {
        eng.scene_base_draw(w);
    }

    // Called when cleanup scene, release assets and resources.
    fn cleanup(&mut self, _eng: &mut Engine, _w: &mut World) {}
}

#[derive(Default)]
pub(crate) struct Perf {
    pub entities: usize,
    pub checks: usize,
    // draw_calls: usize,
    pub update: f32,
    pub draw: f32,
    pub total: f32,
}

pub struct Engine {
    // The real time in seconds since program start
    pub time_real: f32,

    // The game time in seconds since scene start
    pub time: f32,

    // A global multiplier for how fast game time should advance. Default: 1.0
    pub time_scale: f32,

    // The time difference in seconds from the last frame to the current.
    // Typically 0.01666 (assuming 60hz)
    pub tick: f32,

    // The frame number in this current scene. Increases by 1 for every frame.
    pub frame: f32,

    // The map to use for entity vs. world collisions. Reset for each scene.
    // Use engine_set_collision_map() to set it.
    pub collision_map: Option<CollisionMap>,

    // The maps to draw. Reset for each scene. Use engine_add_background_map()
    // to add.
    background_maps: Vec<Map>,

    // A global multiplier that affects the gravity of all entities. This only
    // makes sense for side view games. For a top-down game you'd want to have
    // it at 0.0. Default: 1.0
    pub gravity: f32,

    // Sweep axis
    // The axis (x or y) on which we want to do the broad phase collision detection
    // sweep & prune. For mosly horizontal games it should be x, for vertical ones y
    pub(crate) sweep_axis: SweepAxis,

    // Various infos about the last frame
    pub(crate) perf: Perf,

    // states
    is_running: bool,
    scene: Option<Box<dyn Scene>>,
    scene_next: Option<Box<dyn Scene>>,
    pub(crate) world: UnsafeCell<World>,

    // camera
    pub(crate) camera: Camera,
    // render
    pub(crate) render: RefCell<Render>,
    // input
    pub(crate) input: InputState,
    // commands
    pub(crate) commands: Commands,
    // AssetsManager
    pub assets: AssetManager,
}

impl Engine {
    pub fn new(platform: Box<dyn Platform + 'static>) -> Self {
        Self {
            time_real: 0.0,
            time: 0.0,
            time_scale: 1.0,
            tick: 0.0,
            frame: 0.0,
            collision_map: None,
            background_maps: Default::default(),
            gravity: 0.0,
            camera: Camera::default(),
            perf: Perf::default(),
            is_running: false,
            scene: None,
            scene_next: None,
            world: UnsafeCell::new(Default::default()),
            render: RefCell::new(Render::new(platform)),
            input: InputState::default(),
            commands: Commands::default(),
            sweep_axis: SweepAxis::default(),
            assets: AssetManager::new("assets"),
        }
    }

    pub(crate) fn with_platform<R, F: FnOnce(&mut dyn Platform) -> R>(&mut self, f: F) -> R {
        let mut r = self.render.borrow_mut();
        f(r.platform.as_mut())
    }

    /// Load default font
    pub fn load_default_font<P: AsRef<Path>>(&mut self, path: P) {
        let handle = self.assets.load_font(path);
        self.set_default_font(handle);
    }

    /// Set default font
    pub fn set_default_font(&mut self, handle: Handle) {
        self.render.borrow_mut().set_default_font(handle);
    }

    /// Draw text
    ///
    /// # Arguments
    ///
    /// * `text` - Text
    /// * `pos` - Position
    /// * `anchor` - Anchor, center is (0.5, 0.5), min value (0.0, 0.0), max value (1.0, 1.0)
    /// * `angle` - Angle to rotate
    ///
    /// # Examples
    ///
    /// ```
    /// # use roast2d::prelude::*;
    /// // draw a text with left-top anchor (0, 0)
    /// # fn draw(eng: &mut Engine, w: &mut World, ent: Ent, viewport: Vec2) {
    ///    eng.draw_text(
    ///        Text::new(format!("Hello"), 20.0, BLUE),
    ///        Vec2::new(0.0, 20.0),
    ///        Vec2::ZERO,
    ///        None,
    ///    );
    /// # }
    /// ```
    pub fn draw_text(&mut self, mut text: Text, pos: Vec2, anchor: Vec2, angle: Option<f32>) {
        let w = unsafe { self.borrow_world() };
        let (handle, size) = match w
            .get_resource::<TextCache>()
            .expect("text cache")
            .get(&text)
        {
            Some((handle, size)) => (handle.clone(), *size),
            None => {
                // render text texture
                text.scale *= 2.0;
                let (handle, size) = self.create_text_texture(w, &text);
                let size = size / 2;
                w.get_resource_mut::<TextCache>()
                    .unwrap()
                    .add(text, (handle.clone(), size));
                (handle, size)
            }
        };
        let mut sprite = Sprite::new(handle, size);
        sprite.anchor = anchor;
        self.draw_image(&sprite, pos, None, angle);
    }

    /// Create text texture
    pub fn create_text_texture(&mut self, w: &mut World, text: &Text) -> (Handle, UVec2) {
        let text_cache = w
            .get_resource_mut::<TextCache>()
            .expect("can't get text cache");
        let handle = self.assets.alloc_handle();
        let size = self
            .render
            .borrow_mut()
            .create_text_texture(text_cache, handle.clone(), text);
        (handle, size)
    }

    /// Draw rectangle
    ///
    /// # Arguments
    ///
    /// * `size` - Size
    /// * `pos` - Position
    /// * `color` - Color, color of the rectangle
    /// * `anchor` - Anchor, default is (0.5, 0.5), min value (0.0, 0.0), max value (1.0, 1.0)
    /// * `scale` - Scale
    /// * `angle` - Angle to rotate
    ///
    /// # Examples
    ///
    /// ```
    /// # use roast2d::prelude::*;
    /// // draw a blue rectangle
    /// # fn draw(eng: &mut Engine, w: &mut World, ent: Ent, viewport: Vec2) {
    ///   eng.draw_rect(
    ///       Vec2::splat(40.0),
    ///       Vec2::new(50.0, 100.0) + viewport,
    ///       BLUE,
    ///       None,
    ///       None,
    ///       None,
    ///   );
    /// # }
    /// ```
    pub fn draw_rect(
        &mut self,
        size: Vec2,
        pos: Vec2,
        color: Color,
        anchor: Option<Vec2>,
        scale: Option<Vec2>,
        angle: Option<f32>,
    ) {
        let texture = default_texture(self);
        let mut image = Sprite::with_sizef(texture, size);
        image.color = color;
        if let Some(anchor) = anchor {
            image.anchor = anchor;
        }
        self.render
            .borrow_mut()
            .draw_image(&image, pos, scale, angle);
    }

    /// Draw image
    ///
    /// # Arguments
    ///
    /// * `image` - Sprite to draw
    /// * `pos` - Position
    /// * `scale` - Scale
    /// * `angle` - Angle to rotate
    ///
    /// # Examples
    ///
    /// ```
    /// # use roast2d::prelude::*;
    /// // draw entity's sprite
    /// # fn draw(eng: &mut Engine, w: &mut World, ent: Ent, viewport: Vec2) {
    ///   let ent_ref = w.get(ent).unwrap();
    ///   let sprite = ent_ref.get::<Sprite>().unwrap();
    ///   eng.draw_image(
    ///       &sprite,
    ///       Vec2::new(50.0, 100.0) + viewport,
    ///       None,
    ///       None,
    ///   );
    /// # }
    /// ```
    pub fn draw_image(
        &mut self,
        image: &Sprite,
        pos: Vec2,
        scale: Option<Vec2>,
        angle: Option<f32>,
    ) {
        self.render
            .borrow_mut()
            .draw_image(image, pos, scale, angle);
    }

    /// Draw image as tile
    pub fn draw_tile(
        &mut self,
        image: &Sprite,
        tile: u16,
        tile_size: Vec2,
        dst_pos: Vec2,
        scale: Option<Vec2>,
        angle: Option<f32>,
        flip_x: bool,
        flip_y: bool,
    ) {
        self.render.borrow_mut().draw_tile(
            image, tile, tile_size, dst_pos, scale, angle, flip_x, flip_y,
        );
    }

    /// Sweep axis
    pub fn sweep_axis(&self) -> SweepAxis {
        self.sweep_axis
    }

    /// Set sweep axis
    pub fn set_sweep_axis(&mut self, sweep_axis: SweepAxis) {
        self.sweep_axis = sweep_axis
    }

    /// View size
    pub fn view_size(&self) -> Vec2 {
        self.render.borrow().view_size()
    }

    /// Set view size
    pub fn set_view_size(&mut self, size: Vec2) {
        self.render.borrow_mut().set_view_size(size)
    }

    /// Resize mode
    pub fn resize_mode(&self) -> ResizeMode {
        self.render.borrow().resize_mode()
    }

    /// Set resize mode
    pub fn set_resize_mode(&mut self, mode: ResizeMode) {
        self.render.borrow_mut().set_resize_mode(mode)
    }

    /// Scale mode
    pub fn scale_mode(&self) -> ScaleMode {
        self.render.borrow().scale_mode()
    }

    /// Set scale mode
    pub fn set_scale_mode(&mut self, mode: ScaleMode) {
        self.render.borrow_mut().set_scale_mode(mode)
    }

    // Return seconds since game start
    pub fn now(&mut self) -> f32 {
        self.with_platform(|p| p.now())
    }

    // Input
    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    // Input mut
    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    // Input
    pub fn input(&self) -> &InputState {
        &self.input
    }

    // Input mut
    pub fn input_mut(&mut self) -> &mut InputState {
        &mut self.input
    }

    unsafe fn borrow_world<'a>(&mut self) -> &'a mut World {
        self.world.get().as_mut().unwrap()
    }

    pub(crate) fn init<Setup: FnOnce(&mut Engine, &mut World)>(&mut self, setup: Setup) {
        let world = unsafe { self.borrow_world() };
        self.time_real = self.now();
        // init submodules
        init_collision(self, world);
        // init textcache
        init_text_cache(self, world);
        self.init_default_font(world);

        setup(self, world);
    }

    pub(crate) fn init_default_font(&mut self, world: &mut World) {
        // set default font
        let handle = self.assets.insert(Asset {
            asset_type: AssetType::Font,
            bytes: None,
        });
        let font =
            Font::from_bytes(DEFAULT_FONT_BYTES.into()).expect("Failed to load default font");
        world
            .get_resource_mut::<TextCache>()
            .unwrap()
            .add_font(handle.id(), font);
        self.set_default_font(handle);
    }

    pub(crate) fn resize(&mut self, size: UVec2) {
        self.render.borrow_mut().resize(size);
    }

    /// Scene base draw, draw maps and entities
    pub fn scene_base_draw(&mut self, w: &mut World) {
        let viewport = self.viewport();
        let mut render = self.render.borrow_mut();

        // Background maps
        for map in self.background_maps.iter().rev() {
            if !map.foreground {
                map_draw(&mut render, map, viewport);
            }
        }
        drop(render);

        self.entities_draw(w, viewport);

        // Foreground maps
        let mut render = self.render.borrow_mut();
        for map in self.background_maps.iter().rev() {
            if map.foreground {
                map_draw(&mut render, map, viewport);
            }
        }
    }

    /// Scene base update, update entities
    pub fn scene_base_update(&mut self, w: &mut World) {
        self.entities_update(w);
    }

    fn entities_draw(&mut self, w: &mut World, viewport: Vec2) {
        // Sort entities by draw_order
        let mut ents: Vec<_> = w
            .iter_ent_refs()
            .filter_map(|ent_ref| {
                let transform = ent_ref.get::<Transform>()?;
                Some((ent_ref.id(), transform.z_index))
            })
            .collect();
        ents.sort_by_key(|(_ent, z)| *z);
        for (ent, _z) in ents {
            if let Some(hooks) = get_ent_hooks(w, ent) {
                hooks.draw(self, w, ent, viewport);
            }
        }
    }

    fn entities_update(&mut self, w: &mut World) {
        // Update all entities
        let ents: Vec<_> = w.iter_ents().cloned().collect();
        let ents_count = ents.len();
        for ent in ents {
            let ent_hooks = get_ent_hooks(w, ent);
            if let Some(hooks) = ent_hooks.as_ref() {
                hooks.update(self, w, ent);
            }
            // physics update
            physics::entity_base_update(self, w, ent);
            if let Some(hooks) = ent_hooks {
                hooks.post_update(self, w, ent);
            }
        }

        collision::update_collision(self, w);

        self.perf.entities = ents_count;
    }

    /// Called per frame, the main update logic of engine
    pub(crate) fn update(&mut self) {
        let world = unsafe { self.borrow_world() };
        self.inner_update(world);
    }

    pub(crate) fn inner_update(&mut self, w: &mut World) {
        let time_frame_start = self.now();

        if self.scene_next.is_some() {
            self.is_running = false;
            if let Some(mut scene) = self.scene.take() {
                scene.cleanup(self, w);
            }

            self.background_maps.clear();
            self.collision_map = None;
            self.time = 0.;
            self.frame = 0.;
            self.camera.viewport = Vec2::new(0., 0.);

            if let Some(mut scene) = self.scene_next.take() {
                scene.init(self, w);
                self.scene = Some(scene);
            }
        }
        self.is_running = true;

        let time_real_now = self.now();
        let real_delta = time_real_now - self.time_real;
        self.time_real = time_real_now;
        self.tick = (real_delta * self.time_scale).min(ENGINE_MAX_TICK);
        self.time += self.tick;
        self.frame += 1.;

        if let Some(mut scene) = self.scene.take() {
            scene.update(self, w);
            self.scene = Some(scene);
        } else {
            self.scene_base_update(w);
        }

        // handle_commands
        self.handle_commands(w);

        // Update camera
        let camera_follow = self.camera.follow.and_then(|ent_ref| w.get(ent_ref));
        let bounds = self.collision_map.as_ref().map(|map| map.bounds());
        self.camera.update(
            self.tick,
            self.render.borrow().logical_size(),
            camera_follow,
            bounds,
        );

        self.perf.update = self.now() - time_real_now;

        if let Some(mut scene) = self.scene.take() {
            scene.draw(self, w);
            self.scene = Some(scene);
        } else {
            self.scene_base_draw(w);
        }

        self.perf.draw = (self.now() - time_real_now) - self.perf.update;

        self.input.clear();

        self.perf.total = self.now() - time_frame_start;
    }

    pub(crate) async fn handle_assets(&mut self) -> Result<()> {
        let world = unsafe { self.borrow_world() };

        let tasks = self.assets.fetch().await?;
        for task in tasks {
            match task {
                FetchedTask::CreateTexture { handle, data, size } => {
                    self.with_platform(|p| {
                        p.create_texture(handle, data, size);
                    });
                }
                FetchedTask::RemoveTexture { handle } => {
                    self.with_platform(|p| {
                        p.remove_texture(handle);
                    });
                }
                FetchedTask::CreateFont { handle, font } => {
                    let Some(cache) = world.get_resource_mut::<TextCache>() else {
                        log::error!("Failed to get text cache");
                        continue;
                    };
                    cache.add_font(handle.id(), font);
                }
                FetchedTask::RemoveFont { handle } => {
                    let Some(cache) = world.get_resource_mut::<TextCache>() else {
                        log::error!("Failed to get text cache");
                        continue;
                    };
                    cache.remove_font(handle);
                }
            }
        }
        Ok(())
    }

    pub fn viewport(&self) -> Vec2 {
        let render = self.render.borrow();
        render.snap_px(self.camera.viewport)
    }

    /// Set a scene, the scene swap do not happend instantly, it is happend in engine update
    pub fn set_scene(&mut self, scene: impl Scene + 'static) {
        self.scene_next.replace(Box::new(scene));
    }

    /// Load level
    pub fn load_level(
        &mut self,
        w: &mut World,
        proj: &LdtkProject,
        identifier: &str,
    ) -> Result<()> {
        let level = proj.get_level(identifier)?;
        self.background_maps.clear();
        self.collision_map.take();
        self.commands.take();
        self.input.clear();

        for (index, layer) in level.layer_instances.iter().enumerate() {
            match layer.r#type {
                LayerType::IntGrid if layer.identifier == COLLISION_MAP => {
                    let map = CollisionMap::from_ldtk_layer(layer)?;
                    self.collision_map.replace(map);
                }
                LayerType::AutoLayer | LayerType::Tiles => {
                    let tileset = if let Some(rel_path) = layer.tileset_rel_path.as_ref() {
                        self.assets.load_texture(rel_path)
                    } else {
                        bail!(
                            "Layer {}-{} doesn't has tileset",
                            level.identifier,
                            &layer.identifier
                        )
                    };
                    let map = Map::from_ldtk_layer(proj, level, index, layer, tileset)?;
                    self.background_maps.push(map);
                }
                LayerType::Entities => {
                    // spawn entities
                    for ent_ins in &layer.entity_instances {
                        let pos = Vec2::new(
                            (ent_ins.px.0 + ent_ins.width / 2) as f32,
                            (ent_ins.px.1 + ent_ins.height / 2) as f32,
                        );
                        let ent = {
                            let identifier = &ent_ins.identifier;
                            let mut ent = w.spawn();
                            // add transform
                            ent.add(Transform::new(
                                pos,
                                Vec2::new(ent_ins.width as f32, ent_ins.height as f32),
                            ))
                            // add same name component
                            .add_by_name(identifier);
                            ent.id()
                        };
                        let settings = ent_ins
                            .field_instances
                            .iter()
                            .map(|f| (f.identifier.clone(), f.value.clone()))
                            .collect();
                        self.setting(ent, settings);
                    }
                }
                _ => {
                    log::error!("Ignore layer {} {:?}", layer.identifier, layer.r#type);
                }
            }
        }

        Ok(())
    }

    /// Add background map
    pub fn add_background_map(&mut self, map: Map) {
        self.background_maps.push(map);
    }

    /// Set collision map
    pub fn set_collision_map(&mut self, map: CollisionMap) {
        self.collision_map.replace(map);
    }

    /// Wether the engine is running
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    #[allow(dead_code)]
    pub(crate) fn cleanup(&mut self) {
        // Do nothing
    }

    pub(crate) fn collide(&mut self, ent: Ent, normal: Vec2, trace: Option<Trace>) {
        self.commands.add(Command::Collide { ent, normal, trace });
    }

    /// Setting an entity
    pub fn setting(&mut self, ent: Ent, settings: serde_json::Value) {
        self.commands.add(Command::Setting { ent, settings });
    }

    /// Kill an entity
    pub fn kill(&mut self, ent: Ent) {
        self.commands.add(Command::KillEnt { ent });
    }

    /// Damage an entity
    pub fn damage(&mut self, ent: Ent, by_ent: Ent, damage: f32) {
        self.commands.add(Command::Damage {
            ent,
            by_ent,
            damage,
        });
    }

    /// Trigger an entity
    pub fn trigger(&mut self, ent: Ent, other: Ent) {
        self.commands.add(Command::Trigger { ent, other });
    }

    /// Message an entity
    pub fn message(&mut self, ent: Ent, data: Box<dyn Any>) {
        self.commands.add(Command::Message { ent, data });
    }

    /// Move entity
    pub fn move_ent(&mut self, ent: &mut EntMut, vstep: Vec2) {
        entity_move(self, ent, vstep);
    }

    /// Handle commands
    fn handle_commands(&mut self, w: &mut World) {
        let commands = self.commands.take();
        for command in commands {
            match command {
                Command::Collide { ent, normal, trace } => {
                    if let Some(hooks) = get_ent_hooks(w, ent) {
                        hooks.collide(self, w, ent, normal, trace.as_ref());
                    }
                }
                Command::Setting { ent, settings } => {
                    if let Some(hooks) = get_ent_hooks(w, ent) {
                        hooks.settings(self, w, ent, settings);
                    }
                }
                Command::KillEnt { ent } => {
                    if let Some(mut ent) = w.get_mut(ent) {
                        if let Some(health) = ent.get_mut::<Health>() {
                            health.alive = false;
                        }
                    }
                    if let Some(hooks) = get_ent_hooks(w, ent) {
                        hooks.kill(self, w, ent);
                    }
                    w.despawn(ent);
                }
                Command::Damage {
                    ent,
                    by_ent,
                    damage,
                } => {
                    if let Some(hooks) = get_ent_hooks(w, ent) {
                        hooks.damage(self, w, ent, by_ent, damage);
                    }
                }
                Command::Trigger { ent, other } => {
                    if let Some(hooks) = get_ent_hooks(w, ent) {
                        hooks.trigger(self, w, ent, other);
                    }
                }
                Command::Message { ent, data } => {
                    if let Some(hooks) = get_ent_hooks(w, ent) {
                        hooks.message(self, w, ent, data);
                    }
                }
            }
        }
    }
}
