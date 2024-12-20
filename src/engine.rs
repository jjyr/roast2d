use std::{
    cell::{RefCell, UnsafeCell},
    path::Path,
    sync::OnceLock,
};

use anyhow::Result;
use glam::{UVec2, Vec2};

use crate::{
    asset::{Asset, AssetManager, AssetType, FetchedTask},
    camera::Camera,
    color::Color,
    ecs::world::World,
    font::{Font, Text},
    handle::Handle,
    input::InputState,
    platform::Platform,
    render::{Render, ScaleMode},
    sprite::Sprite,
    text_cache::{init_text_cache, TextCache},
};

/// Default texture
static DEFAULT_TEXTURE: OnceLock<Handle> = OnceLock::new();
/// Max tick
const ENGINE_MAX_TICK: f32 = 100.0;
/// Default font
const DEFAULT_FONT_BYTES: &[u8; 59164] = include_bytes!("../assets/Pixel Square 10.ttf");

/// get default texture
fn default_texture(g: &mut Engine) -> Handle {
    DEFAULT_TEXTURE
        .get_or_init(|| {
            let data = vec![255, 255, 255, 255];
            let size = UVec2::splat(1);
            let handle = g.assets.insert(Asset {
                asset_type: AssetType::Texture,
                bytes: None,
            });
            g.with_platform(|p| {
                p.create_texture(handle.clone(), data, size);
            });
            handle
        })
        .clone()
}

// Scene trait
pub trait Scene {
    // Init the scene, use it to load assets and setup entities.
    fn init(&mut self, _g: &mut Engine, _w: &mut World);

    // Update scene per frame, you probably want to call scene_base_update if you override this function.
    fn update(&mut self, g: &mut Engine, w: &mut World);

    // Draw scene per frame, use it to draw entities or Hud, you probably want to call scene_base_draw if you override this function.
    fn draw(&mut self, g: &mut Engine, w: &mut World);

    // Called when cleanup scene, release assets and resources.
    fn cleanup(&mut self, _g: &mut Engine, _w: &mut World);
}

#[derive(Default, Debug)]
pub struct Perf {
    pub entities: usize,
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

    // Camera bounds
    pub bounds: Option<Vec2>,

    // A global multiplier that affects the gravity of all entities. This only
    // makes sense for side view games. For a top-down game you'd want to have
    // it at 0.0. Default: 1.0
    pub gravity: f32,

    // Various infos about the last frame
    pub perf: Perf,

    // input
    pub input: InputState,

    // states
    is_running: bool,
    is_window_resized: bool,
    scene: Option<Box<dyn Scene>>,
    scene_next: Option<Box<dyn Scene>>,
    pub(crate) world: UnsafeCell<World>,

    // camera
    pub(crate) camera: Camera,
    // render
    pub(crate) render: RefCell<Render>,
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
            bounds: None,
            gravity: 0.0,
            camera: Camera::default(),
            perf: Perf::default(),
            is_running: false,
            is_window_resized: false,
            scene: None,
            scene_next: None,
            world: UnsafeCell::new(Default::default()),
            render: RefCell::new(Render::new(platform)),
            input: InputState::default(),
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
    /// # fn draw(g: &mut Engine, w: &mut World, ent: Ent, viewport: Vec2) {
    ///    g.draw_text(
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
        let handle = self.assets.insert(Asset {
            asset_type: AssetType::Texture,
            bytes: None,
        });
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
    /// # fn draw(g: &mut Engine, w: &mut World, ent: Ent, viewport: Vec2) {
    ///   g.draw_rect(
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
    /// # fn draw(g: &mut Engine, w: &mut World, ent: Ent, viewport: Vec2) {
    ///   let ent_ref = w.get(ent).unwrap();
    ///   let sprite = ent_ref.get::<Sprite>().unwrap();
    ///   g.draw_image(
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

    /// View size
    pub fn view_size(&self) -> Vec2 {
        self.render.borrow().logical_size()
    }

    pub fn is_window_resized(&self) -> bool {
        self.is_window_resized
    }

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

    #[allow(dead_code)]
    pub(crate) fn on_resize(&mut self, size: UVec2) {
        self.render.borrow_mut().resize(size);
        self.is_window_resized = true;
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
        }
        self.perf.entities = w.ents_count();

        // Update camera
        let camera_follow = self.camera.follow.and_then(|ent_ref| w.get(ent_ref).ok());
        self.camera.update(
            self.tick,
            self.render.borrow().logical_size(),
            camera_follow,
            self.bounds,
        );
        self.perf.update = self.now() - time_real_now;

        if let Some(mut scene) = self.scene.take() {
            scene.draw(self, w);
            self.scene = Some(scene);
        }

        self.perf.draw = (self.now() - time_real_now) - self.perf.update;
        self.input.clear();
        self.is_window_resized = false;
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
                    let Ok(cache) = world.get_resource_mut::<TextCache>() else {
                        log::error!("Failed to get text cache");
                        continue;
                    };
                    cache.add_font(handle.id(), font);
                }
                FetchedTask::RemoveFont { handle } => {
                    let Ok(cache) = world.get_resource_mut::<TextCache>() else {
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

    /// Set collision map
    pub fn set_bounds(&mut self, bounds: Vec2) {
        self.bounds.replace(bounds);
    }

    /// Wether the engine is running
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    #[allow(dead_code)]
    pub(crate) fn cleanup(&mut self) {
        // Do nothing
    }
}
