use std::{
    any::{type_name, Any},
    cell::{RefCell, RefMut},
    rc::Rc,
};

use anyhow::{bail, Result};
use glam::{UVec2, Vec2};

use crate::{
    asset::{AssetManager, FetchedTask},
    camera::Camera,
    collision_map::{CollisionMap, COLLISION_MAP},
    commands::{Command, Commands},
    entity::{
        entity_move, resolve_collision, with_ent, Ent, EntCollidesMode, EntPhysics, EntRef,
        EntType, EntTypeId,
    },
    font::Text,
    handle::Handle,
    input::InputState,
    ldtk::{LayerType, LdtkProject},
    map::{map_draw, Map},
    platform::Platform,
    render::{Render, ResizeMode, ScaleMode},
    sprite::Sprite,
    trace::Trace,
    types::SweepAxis,
    world::World,
};

const ENGINE_MAX_TICK: f32 = 100.0;

// Scene trait
pub trait Scene {
    // Init the scene, use it to load assets and setup entities.
    fn init(&mut self, _eng: &mut Engine) {}

    // Update scene per frame, you probably want to call scene_base_update if you override this function.
    fn update(&mut self, eng: &mut Engine) {
        eng.scene_base_update();
    }

    // Draw scene per frame, use it to draw entities or Hud, you probably want to call scene_base_draw if you override this function.
    fn draw(&mut self, eng: &mut Engine) {
        eng.scene_base_draw();
    }

    // Called when cleanup scene, release assets and resources.
    fn cleanup(&mut self, _eng: &mut Engine) {}
}

#[derive(Default)]
pub struct Perf {
    entities: usize,
    checks: usize,
    // draw_calls: usize,
    update: f32,
    draw: f32,
    total: f32,
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
    sweep_axis: SweepAxis,

    // Various infos about the last frame
    perf: Perf,

    // states
    is_running: bool,
    scene: Option<Box<dyn Scene>>,
    scene_next: Option<Box<dyn Scene>>,
    pub(crate) world: Rc<RefCell<World>>,

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
            world: Rc::new(RefCell::new(Default::default())),
            render: RefCell::new(Render::new(platform)),
            input: InputState::default(),
            commands: Commands::default(),
            sweep_axis: SweepAxis::default(),
            assets: AssetManager::new("assets"),
        }
    }

    /// Registry a new entity type
    /// this function must be called before add entity
    pub fn add_ent_type<T: EntType + Clone + 'static>(&mut self) {
        let ent_type: Rc<dyn EntType> = Rc::new(T::load(self));
        self.world.borrow_mut().add_ent_type::<T>(ent_type);
    }

    /// Spawn a new entity
    pub fn spawn<T: EntType + 'static>(&mut self, pos: Vec2) -> EntRef {
        // fetch id
        let ent_type = EntTypeId::of::<T>();
        match self.spawn_with_type_id(ent_type, pos) {
            Some(ent_ref) => ent_ref,
            None => {
                panic!(
                    "Can't get entity type, make sure {} is registered with add_entity_type",
                    type_name::<T>()
                )
            }
        }
    }

    pub(crate) fn with_platform<R, F: FnOnce(&mut dyn Platform) -> R>(&mut self, f: F) -> R {
        let mut r = self.render.borrow_mut();
        f(r.platform.as_mut())
    }

    pub fn spawn_with_type_name(&mut self, name: String, pos: Vec2) -> EntRef {
        match self
            .with_world(|_eng: &mut Engine, w: RefMut<World>| w.get_type_id_by_name(&name).cloned())
            .and_then(|ent_type_id| self.spawn_with_type_id(ent_type_id, pos))
        {
            Some(ent_ref) => ent_ref,
            None => {
                panic!("Can't get entity type, make sure {name} is registered with add_entity_type")
            }
        }
    }

    pub fn spawn_with_type_id(&mut self, ent_type: EntTypeId, pos: Vec2) -> Option<EntRef> {
        let ent_ref = {
            let mut world = self.world.borrow_mut();
            let instance = world.get_ent_type_instance(&ent_type)?;
            let id = world.next_unique_id();

            // add to world
            let ent = Ent::new(id, ent_type, instance, pos);
            world.spawn(ent)
        };

        // init
        with_ent(self, ent_ref, |eng, ent_ref, instance: &mut dyn EntType| {
            instance.init(eng, ent_ref);
        });

        Some(ent_ref)
    }

    /// Create text
    pub fn create_text_texture(&mut self, text: Text) -> (Handle, UVec2) {
        let handle = self.assets.alloc_handle();
        let size = self
            .render
            .borrow_mut()
            .create_text_texture(handle.clone(), text);
        (handle, size)
    }

    /// Draw image
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

    // World
    pub fn with_world<R, F: FnOnce(&mut Engine, RefMut<World>) -> R>(&mut self, f: F) -> R {
        let world = self.world.clone();
        f(self, world.as_ref().borrow_mut())
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

    pub(crate) fn init(&mut self) {
        self.time_real = self.now();
    }

    /// Scene base draw, draw maps and entities
    pub fn scene_base_draw(&mut self) {
        let mut render = self.render.borrow_mut();
        let px_viewport = render.snap_px(self.camera.viewport);

        // Background maps
        for map in self.background_maps.iter().rev() {
            if !map.foreground {
                map_draw(&mut render, map, px_viewport);
            }
        }
        drop(render);

        self.entities_draw(px_viewport);

        // Foreground maps
        let mut render = self.render.borrow_mut();
        for map in self.background_maps.iter().rev() {
            if map.foreground {
                map_draw(&mut render, map, px_viewport);
            }
        }
    }

    /// Scene base update, update entities
    pub fn scene_base_update(&mut self) {
        self.entities_update();
    }

    /// Entity base update, handle physics velocities
    pub(crate) fn entity_base_update(&mut self, ent: EntRef) {
        self.with_world(|eng, mut world| {
            let Some(ent) = world.get_mut(ent) else {
                return;
            };
            if !ent.physics.contains(EntPhysics::MOVE) {
                return;
            }
            // Integrate velocity
            let vel = ent.vel;
            ent.vel.y += eng.gravity * ent.gravity * eng.tick;
            let fric = Vec2::new(
                (ent.friction.x * eng.tick).min(1.0),
                (ent.friction.y * eng.tick).min(1.0),
            );
            ent.vel = ent.vel + (ent.accel * eng.tick - ent.vel * fric);
            let vstep = (vel + ent.vel) * (eng.tick * 0.5);
            ent.on_ground = false;
            entity_move(eng, ent, vstep);
        });
    }

    fn entities_draw(&mut self, viewport: Vec2) {
        // Sort entities by draw_order
        let mut ents: Vec<_> = self.with_world(|_eng: &mut Engine, world: RefMut<World>| {
            world.entities().map(|ent| ent.ent_ref).collect()
        });
        ents.sort_by_key(|ent_ref| {
            self.with_world(|_: &mut Engine, world: RefMut<World>| {
                world.get(*ent_ref).unwrap().draw_order
            })
        });
        for ent_ref in ents {
            with_ent(
                self,
                ent_ref,
                |eng: &mut Engine, ent_ref: EntRef, instance: &mut dyn EntType| {
                    instance.draw(eng, ent_ref, viewport);
                },
            );
        }
    }

    fn entities_update(&mut self) {
        // Update all entities
        let mut i = 0;
        while i < self.with_world(|_, w: RefMut<World>| w.alloced()) {
            let ent_ref = self.with_world(|_, w: _| w.get_nth(i).cloned().unwrap());
            with_ent(
                self,
                ent_ref,
                |eng: &mut Engine, ent_ref: EntRef, instance: &mut dyn EntType| {
                    instance.update(eng, ent_ref);
                },
            );
            self.entity_base_update(ent_ref);
            with_ent(
                self,
                ent_ref,
                |eng: &mut Engine, ent_ref, instance: &mut dyn EntType| {
                    instance.post_update(eng, ent_ref);
                },
            );

            self.with_world(|_, mut w| {
                if w.get_unchecked(ent_ref).is_some_and(|ent| !ent.alive) {
                    // remove if not alive
                    w.swap_remove(i);
                }
            });

            i += 1;
        }

        // Sort by x or y position
        // insertion sort can gain better performance since list is sorted in every frames
        let sweep_axis = self.sweep_axis;
        self.with_world(|_, mut w| w.sort_entities_for_sweep(sweep_axis));

        // Sweep touches
        self.perf.checks = 0;
        let entities_count = self.with_world(|_, w: _| w.alloced());
        for i in 0..entities_count {
            let (ent1, ent1_bounds) = self.with_world(|_, w| {
                let ent1 = w.get_nth(i).cloned().unwrap();
                let ent1_bounds = w.get(ent1).unwrap().bounds();
                (ent1, ent1_bounds)
            });
            let res = self.with_world(|_, w| {
                let ent1 = w.get(ent1).unwrap();
                !ent1.check_against.is_empty()
                    || !ent1.group.is_empty()
                    || ent1.physics.is_at_least(EntPhysics::PASSIVE)
            });
            if res {
                let max_pos = sweep_axis.get(ent1_bounds.max);
                for j in (i + 1)..entities_count {
                    let (ent2, ent2_bounds) = self.with_world(|_, w| {
                        let ent2 = w.get_nth(j).cloned().unwrap();
                        let ent2_bounds = w.get(ent2).unwrap().bounds();
                        (ent2, ent2_bounds)
                    });
                    if sweep_axis.get(ent2_bounds.min) > max_pos {
                        break;
                    }
                    self.perf.checks += 1;
                    if ent1_bounds.is_touching(&ent2_bounds) {
                        let res = self.with_world(|_, w| {
                            let [ent1, ent2] = w.many([ent1, ent2]);
                            !(ent1.check_against & ent2.group).is_empty()
                        });
                        if res {
                            with_ent(
                                self,
                                ent1,
                                |eng: &mut Engine, ent1: EntRef, instance: &mut dyn EntType| {
                                    instance.touch(eng, ent1, ent2);
                                },
                            );
                        }
                        let res = self.with_world(|_, w| {
                            let [ent1, ent2] = w.many([ent1, ent2]);
                            !(ent1.group & ent2.check_against).is_empty()
                        });
                        if res {
                            with_ent(
                                self,
                                ent2,
                                |eng: &mut Engine, ent2: EntRef, instance: &mut dyn EntType| {
                                    instance.touch(eng, ent2, ent1);
                                },
                            );
                        }

                        let res = self.with_world(|_, w| {
                            let [ent1, ent2] = w.many([ent1, ent2]);
                            ent1.physics.bits() >= EntCollidesMode::LITE.bits()
                                && ent2.physics.bits() >= EntCollidesMode::LITE.bits()
                                && ent1.physics.bits().saturating_add(ent2.physics.bits())
                                    >= (EntCollidesMode::ACTIVE | EntCollidesMode::LITE).bits()
                                && (ent1.mass + ent2.mass) > 0.0
                        });
                        if res {
                            resolve_collision(self, ent1, ent2);
                        }
                    }
                }
            }
        }
        self.perf.entities = self.with_world(|_, w| w.alloced());
    }

    /// Called per frame, the main update logic of engine
    pub(crate) fn update(&mut self) {
        let time_frame_start = self.now();

        if self.scene_next.is_some() {
            self.is_running = false;
            if let Some(mut scene) = self.scene.take() {
                scene.cleanup(self);
            }

            self.with_world(|_, mut w| {
                w.reset_entities();
            });

            self.background_maps.clear();
            self.collision_map = None;
            self.time = 0.;
            self.frame = 0.;
            self.camera.viewport = Vec2::new(0., 0.);

            if let Some(mut scene) = self.scene_next.take() {
                scene.init(self);
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
            scene.update(self);
            self.scene = Some(scene);
        } else {
            self.scene_base_update();
        }

        // handle_commands
        self.handle_commands();

        // Update camera
        self.with_world(|eng, w| {
            let camera_follow = eng.camera.follow.and_then(|ent_ref| w.get(ent_ref));
            let bounds = eng.collision_map.as_ref().map(|map| map.bounds());
            eng.camera.update(
                eng.tick,
                eng.render.borrow().logical_size(),
                camera_follow,
                bounds,
            );
        });

        self.perf.update = self.now() - time_real_now;

        if let Some(mut scene) = self.scene.take() {
            scene.draw(self);
            self.scene = Some(scene);
        } else {
            self.scene_base_draw();
        }

        self.perf.draw = (self.now() - time_real_now) - self.perf.update;

        self.input.clear();

        self.perf.total = self.now() - time_frame_start;
    }

    pub(crate) async fn handle_assets(&mut self) -> Result<()> {
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
            }
        }
        Ok(())
    }

    /// Set a scene, the scene swap do not happend instantly, it is happend in engine update
    pub fn set_scene(&mut self, scene: impl Scene + 'static) {
        self.scene_next.replace(Box::new(scene));
    }

    /// Load level
    pub fn load_level(&mut self, proj: &LdtkProject, identifier: &str) -> Result<()> {
        let level = proj.get_level(identifier)?;
        self.with_world(|_, mut w| {
            w.reset_entities();
        });
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
                    for ent_ins in &layer.entity_instances {
                        let pos = Vec2::new(
                            (ent_ins.px.0 + ent_ins.width / 2) as f32,
                            (ent_ins.px.1 + ent_ins.height / 2) as f32,
                        );
                        let ent_ref = self.spawn_with_type_name(ent_ins.identifier.clone(), pos);
                        self.with_world(|_, mut w| {
                            if let Some(ent) = w.get_mut(ent_ref) {
                                ent.size.x = ent_ins.width as f32;
                                ent.size.y = ent_ins.height as f32;
                            }
                        });
                        let settings = ent_ins
                            .field_instances
                            .iter()
                            .map(|f| (f.identifier.clone(), f.value.clone()))
                            .collect();

                        self.setting(ent_ref, settings);
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

    pub(crate) fn collide(&mut self, ent: EntRef, normal: Vec2, trace: Option<Trace>) {
        self.commands.add(Command::Collide { ent, normal, trace });
    }

    /// Setting an entity
    pub fn setting(&mut self, ent: EntRef, settings: serde_json::Value) {
        self.commands.add(Command::Setting { ent, settings });
    }

    /// Kill an entity
    pub fn kill(&mut self, ent: EntRef) {
        self.commands.add(Command::KillEntity { ent });
    }

    /// Damage an entity
    pub fn damage(&mut self, ent: EntRef, by_ent: EntRef, damage: f32) {
        self.commands.add(Command::Damage {
            ent,
            by_ent,
            damage,
        });
    }

    /// Trigger an entity
    pub fn trigger(&mut self, ent: EntRef, other: EntRef) {
        self.commands.add(Command::Trigger { ent, other });
    }

    /// Message an entity
    pub fn message(&mut self, ent: EntRef, data: Box<dyn Any>) {
        self.commands.add(Command::Message { ent, data });
    }

    /// Move entity
    pub fn move_ent(&mut self, ent: &mut Ent, vstep: Vec2) {
        entity_move(self, ent, vstep);
    }

    pub(crate) fn handle_commands(&mut self) {
        let commands = self.commands.take();
        for command in commands {
            match command {
                Command::Collide { ent, normal, trace } => {
                    with_ent(
                        self,
                        ent,
                        |eng: &mut Engine, ent_ref: EntRef, instance: &mut dyn EntType| {
                            instance.collide(eng, ent_ref, normal, trace.as_ref());
                        },
                    );
                }
                Command::Setting { ent, settings } => {
                    with_ent(
                        self,
                        ent,
                        |eng: &mut Engine, ent_ref: EntRef, instance: &mut dyn EntType| {
                            instance.settings(eng, ent_ref, settings);
                        },
                    );
                }
                Command::KillEntity { ent } => {
                    self.with_world(|_, mut w| {
                        if let Some(ent) = w.get_mut(ent) {
                            ent.alive = false;
                        }
                    });
                    with_ent(self, ent, |eng, ent, instance| {
                        instance.kill(eng, ent);
                    });
                }
                Command::Damage {
                    ent,
                    by_ent,
                    damage,
                } => {
                    with_ent(self, ent, |eng, ent, instance| {
                        instance.damage(eng, ent, by_ent, damage);
                    });
                }
                Command::Trigger { ent, other } => {
                    with_ent(self, ent, |eng, ent, instance| {
                        instance.trigger(eng, ent, other);
                    });
                }
                Command::Message { ent, data } => {
                    with_ent(self, ent, |eng, ent, instance| {
                        instance.message(eng, ent, data);
                    });
                }
            }
        }
    }
}
