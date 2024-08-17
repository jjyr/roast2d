use std::{
    any::{type_name, Any},
    path::Path,
    time::Instant,
};

use anyhow::Result;
use glam::Vec2;

use crate::{
    camera::Camera,
    collision_map::{CollisionMap, COLLISION_MAP},
    commands::{Command, Commands},
    entity::{
        entity_move, resolve_collision, with_ent, Entity, EntityCollidesMode, EntityPhysics,
        EntityRef, EntityType, EntityTypeId, World,
    },
    font::Text,
    image::Image,
    input::InputState,
    ldtk::{LayerType, LdtkProject},
    map::{map_draw, Map},
    render::{Render, ResizeMode, ScaleMode},
    sorts::insertion_sort_by_key,
    trace::Trace,
    types::SweepAxis,
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
    world: World,

    // camera
    pub(crate) camera: Camera,
    // render
    pub(crate) render: Render,
    // input
    pub(crate) input: InputState,
    // commands
    pub(crate) commands: Commands,

    // time
    start_time: Instant,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
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
            world: Default::default(),
            render: Render::default(),
            start_time: Instant::now(),
            input: InputState::default(),
            commands: Commands::default(),
            sweep_axis: SweepAxis::default(),
        }
    }

    /// Registry a new entity type
    /// this function must be called before add entity
    pub fn add_entity_type<T: EntityType + Default + Clone + 'static>(&mut self) {
        self.world.add_entity_type::<T>()
    }

    /// Spawn a new entity
    pub fn spawn<T: EntityType + 'static>(&mut self, pos: Vec2) -> EntityRef {
        // fetch id
        let ent_type = EntityTypeId::of::<T>();
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

    pub fn spawn_with_type_name(&mut self, name: String, pos: Vec2) -> EntityRef {
        match self
            .world
            .name_to_entity_types
            .get(&name)
            .cloned()
            .and_then(|ent_type| self.spawn_with_type_id(ent_type, pos))
        {
            Some(ent_ref) => ent_ref,
            None => {
                panic!("Can't get entity type, make sure {name} is registered with add_entity_type")
            }
        }
    }

    pub fn spawn_with_type_id(&mut self, ent_type: EntityTypeId, pos: Vec2) -> Option<EntityRef> {
        let instance = self.world.get_entity_type_instance(&ent_type)?;

        let id = self.world.unique_id;
        // init
        let mut ent = Entity::new(id, ent_type, instance, pos);
        with_ent!(ent, |instance: &mut Box<dyn EntityType>| {
            instance.init(self, &mut ent);
        });

        // add to world
        Some(self.world.spawn(ent))
    }

    /// Load image from path
    pub fn load_image<P: AsRef<Path>>(&self, path: P) -> Result<Image> {
        self.render.load_image(path)
    }

    /// Create text
    pub fn create_text_texture(&self, text: Text) -> Result<Image> {
        self.render.create_text_texture(text)
    }

    /// Draw image
    pub fn draw_image(&mut self, image: &Image, pos: Vec2) {
        let size = image.size();
        image.draw_tile_ex(
            &mut self.render,
            0,
            Vec2::new(size.x as f32, size.y as f32),
            pos,
            false,
            false,
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
        self.render.view_size()
    }

    /// Set view size
    pub fn set_view_size(&mut self, size: Vec2) {
        self.render.set_view_size(size)
    }

    /// VSync
    pub fn vsync(&self) -> bool {
        self.render.vsync
    }

    /// Set view size
    pub fn set_vsync(&mut self, vsync: bool) {
        self.render.vsync = vsync;
    }

    /// Resize mode
    pub fn resize_mode(&self) -> ResizeMode {
        self.render.resize_mode()
    }

    /// Set resize mode
    pub fn set_resize_mode(&mut self, mode: ResizeMode) {
        self.render.set_resize_mode(mode)
    }

    /// Scale mode
    pub fn scale_mode(&self) -> ScaleMode {
        self.render.scale_mode()
    }

    /// Set scale mode
    pub fn set_scale_mode(&mut self, mode: ScaleMode) {
        self.render.set_scale_mode(mode)
    }

    // Return seconds since game start
    pub fn now(&self) -> f32 {
        self.start_time.elapsed().as_secs_f32()
    }

    // World
    pub fn world(&self) -> &World {
        &self.world
    }

    // World mut
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
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
        // sound_init(platform_samplerate());
        // platform_set_audio_mix_cb(sound_mix_stereo);
        // main_init();
    }

    /// Scene base draw, draw maps and entities
    pub fn scene_base_draw(&mut self) {
        let px_viewport = self.render.render_snap_px(self.camera.viewport);

        // Background maps
        for map in &self.background_maps {
            if !map.foreground {
                map_draw(&mut self.render, map, px_viewport);
            }
        }

        self.entities_draw(px_viewport);

        // Foreground maps
        for map in &self.background_maps {
            if !map.foreground {
                map_draw(&mut self.render, map, px_viewport);
            }
        }
    }

    /// Scene base update, update entities
    pub fn scene_base_update(&mut self) {
        self.entities_update();
    }

    /// Entity base update, handle physics velocities
    pub fn entity_base_update(&mut self, ent: &mut Entity) {
        if !ent.physics.contains(EntityPhysics::MOVE) {
            return;
        }
        // Integrate velocity
        let vel = ent.vel;
        ent.vel.y += self.gravity * ent.gravity * self.tick;
        let fric = Vec2::new(
            (ent.friction.x * self.tick).min(1.0),
            (ent.friction.y * self.tick).min(1.0),
        );
        ent.vel = ent.vel + (ent.accel * self.tick - ent.vel * fric);
        let vstep = (vel + ent.vel) * (self.tick * 0.5);
        ent.on_ground = false;
        entity_move(self, ent, vstep);
    }

    fn entities_draw(&mut self, viewport: Vec2) {
        // Sort entities by draw_order
        let mut ents = self.world.entities.clone();
        ents.sort_by_key(|ent| ent.borrow().draw_order);
        for ent in ents.iter_mut() {
            let mut ent = ent.borrow_mut();
            with_ent!(ent, |instance: &mut Box<dyn EntityType>| {
                instance.draw(self, &mut ent, viewport);
            });
        }
    }

    fn entities_update(&mut self) {
        // Update all entities
        let mut i = 0;
        while i < self.world.alloced {
            let ent = self.world.entities[i].clone();
            let mut ent = ent.borrow_mut();
            with_ent!(ent, |instance: &mut Box<dyn EntityType>| {
                instance.update(self, &mut ent);
            });

            if !ent.alive {
                // remove if not alive
                self.world.entities.swap_remove(i);
                self.world.alloced -= 1;
            }
            i += 1;
        }

        // Sort by x or y position
        // insertion sort can gain better performance since list is sorted in every frames
        let sweep_axis = self.sweep_axis;
        insertion_sort_by_key(&mut self.world.entities, |ent| {
            sweep_axis.get(ent.borrow().pos) as usize
        });

        // Sweep touches
        self.perf.checks = 0;
        for i in 0..self.world.entities.len() {
            let ent1 = self.world.entities[i].clone();
            let res = {
                let ent1 = ent1.borrow();
                !ent1.check_against.is_empty()
                    || !ent1.group.is_empty()
                    || ent1.physics.is_at_least(EntityPhysics::PASSIVE)
            };
            if res {
                let max_pos = {
                    let ent1 = ent1.borrow();
                    sweep_axis.get(ent1.pos) + sweep_axis.get(ent1.size)
                };
                for j in (i + 1)..self.world.entities.len() {
                    let ent1 = self.world.entities[i].clone();
                    let ent2 = self.world.entities[j].clone();
                    if sweep_axis.get(ent2.borrow().pos) > max_pos {
                        break;
                    }
                    self.perf.checks += 1;
                    if ent1.borrow().is_touching(&ent2.borrow()) {
                        if !(ent1.borrow().check_against & ent2.borrow().group).is_empty() {
                            let mut ent1 = ent1.borrow_mut();
                            with_ent!(ent1, |instance: &mut Box<dyn EntityType>| {
                                instance.touch(self, &mut ent1, &mut ent2.borrow_mut());
                            });
                        }
                        if !(ent1.borrow().group & ent2.borrow().check_against).is_empty() {
                            let mut ent2 = ent2.borrow_mut();
                            with_ent!(ent2, |instance: &mut Box<dyn EntityType>| {
                                instance.touch(self, &mut ent2, &mut ent1.borrow_mut());
                            });
                        }

                        let res = {
                            let ent1 = ent1.borrow();
                            let ent2 = ent2.borrow();
                            ent1.physics.bits() >= EntityCollidesMode::LITE.bits()
                                && ent2.physics.bits() >= EntityCollidesMode::LITE.bits()
                                && ent1.physics.bits().saturating_add(ent2.physics.bits())
                                    >= (EntityCollidesMode::ACTIVE | EntityCollidesMode::LITE)
                                        .bits()
                                && (ent1.mass + ent2.mass) > 0.0
                        };
                        if res {
                            resolve_collision(self, &mut ent1.borrow_mut(), &mut ent2.borrow_mut());
                        }
                    }
                }
            }
        }
        self.perf.entities = self.world.entities.len();
    }

    /// Called per frame, the main update logic of engine
    pub(crate) fn update(&mut self) {
        let time_frame_start = self.now();

        if self.scene_next.is_some() {
            self.is_running = false;
            if let Some(mut scene) = self.scene.take() {
                scene.cleanup(self);
            }

            self.world.entities.clear();

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
        let camera_follow = self
            .camera
            .follow
            .and_then(|ent_ref| self.world.get(ent_ref));
        let bounds = self.collision_map.as_ref().map(|map| map.bounds());
        self.camera
            .update(self.tick, self.render.logical_size(), camera_follow, bounds);

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

    /// Set a scene, the scene swap do not happend instantly, it is happend in engine update
    pub fn set_scene(&mut self, scene: impl Scene + 'static) {
        self.scene_next.replace(Box::new(scene));
    }

    /// Load level
    pub fn load_level(&mut self, proj: &LdtkProject, identifier: &str) -> Result<()> {
        let level = proj.get_level(identifier)?;
        self.world.reset_entities();
        self.background_maps.clear();
        self.collision_map.take();

        for (index, layer) in level.layer_instances.iter().enumerate() {
            match layer.r#type {
                LayerType::IntGrid if layer.identifier == COLLISION_MAP => {
                    let map = CollisionMap::from_ldtk_layer(layer)?;
                    self.collision_map.replace(map);
                }
                LayerType::AutoLayer => {
                    let map = Map::from_ldtk_layer(proj, level, index, layer, &mut self.render)?;
                    self.background_maps.push(map);
                }
                LayerType::Entities => {
                    for ent in &layer.entity_instances {
                        let pos = Vec2::new(ent.px.0 as f32, ent.px.1 as f32);
                        let ent_ref = self.spawn_with_type_name(ent.identifier.clone(), pos);
                        let settings = ent
                            .field_instances
                            .iter()
                            .map(|f| (f.identifier.clone(), f.value.clone()))
                            .collect();

                        self.setting(ent_ref, settings);
                    }
                }
                _ => {
                    eprintln!(
                        "Ignore layer {}, because read from {:?} is unimplemented",
                        layer.identifier, layer.r#type
                    );
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

    pub(crate) fn cleanup(&mut self) {
        // Do nothing
    }

    pub(crate) fn collide(&mut self, ent: &mut Entity, normal: Vec2, trace: Option<&Trace>) {
        with_ent!(ent, |instance: &mut Box<dyn EntityType>| {
            instance.collide(self, ent, normal, trace);
        });
    }

    /// Setting an entity
    pub fn setting(&mut self, ent: EntityRef, settings: serde_json::Value) {
        self.commands.add(Command::Setting { ent, settings });
    }

    /// Kill an entity
    pub fn kill(&mut self, ent: EntityRef) {
        self.commands.add(Command::KillEntity { ent });
    }

    /// Damage an entity
    pub fn damage(&mut self, ent: EntityRef, by_ent: EntityRef, damage: f32) {
        self.commands.add(Command::Damage {
            ent,
            by_ent,
            damage,
        });
    }

    /// Trigger an entity
    pub fn trigger(&mut self, ent: EntityRef, other: EntityRef) {
        self.commands.add(Command::Trigger { ent, other });
    }

    /// Message an entity
    pub fn message(&mut self, ent: EntityRef, msg_id: u32, data: Box<dyn Any>) {
        self.commands.add(Command::Message { ent, msg_id, data });
    }

    pub(crate) fn handle_commands(&mut self) {
        let commands = self.commands.take();
        for command in commands {
            match command {
                Command::Setting { ent, settings } => {
                    let Some(ent) = self.world.get(ent) else {
                        continue;
                    };

                    let mut ent = ent.borrow_mut();
                    with_ent!(ent, |instance: &mut Box<dyn EntityType>| {
                        instance.settings(self, &mut ent, settings);
                    });
                }
                Command::KillEntity { ent } => {
                    let Some(ent) = self.world.get(ent) else {
                        continue;
                    };

                    let mut ent = ent.borrow_mut();
                    ent.alive = false;
                    with_ent!(ent, |instance: &mut Box<dyn EntityType>| {
                        instance.kill(self, &mut ent);
                    });
                }
                Command::Damage {
                    ent,
                    by_ent,
                    damage,
                } => {
                    let Some(ent) = self.world.get(ent) else {
                        continue;
                    };
                    let Some(by_ent) = self.world.get(by_ent) else {
                        continue;
                    };

                    let mut ent = ent.borrow_mut();
                    let mut other = by_ent.borrow_mut();

                    with_ent!(ent, |instance: &mut Box<dyn EntityType>| {
                        instance.damage(self, &mut ent, &mut other, damage);
                    });
                }
                Command::Trigger { ent, other } => {
                    let Some(ent) = self.world.get(ent) else {
                        continue;
                    };
                    let Some(other) = self.world.get(other) else {
                        continue;
                    };

                    let mut ent = ent.borrow_mut();
                    let mut other = other.borrow_mut();

                    with_ent!(ent, |instance: &mut Box<dyn EntityType>| {
                        instance.trigger(self, &mut ent, &mut other);
                    });
                }
                Command::Message { ent, msg_id, data } => {
                    let Some(ent) = self.world.get(ent) else {
                        continue;
                    };

                    let mut ent = ent.borrow_mut();

                    with_ent!(ent, |instance: &mut Box<dyn EntityType>| {
                        instance.message(self, &mut ent, msg_id, data);
                    });
                }
            }
        }
    }
}
