use anyhow::Result;
use glam::UVec2;

use crate::{
    color::Color,
    ecs::world::World,
    engine::Engine,
    handle::{Handle, HandleId},
    types::Rect,
};

#[cfg(not(target_arch = "wasm32"))]
mod sdl;
#[cfg(target_arch = "wasm32")]
mod web;

pub trait Platform {
    /// Return seconds since game started
    fn now(&mut self) -> f32;
    fn prepare_frame(&mut self);
    fn end_frame(&mut self);
    fn cleanup(&mut self);
    fn draw(
        &mut self,
        texture: &Handle,
        color: Color,
        src: Option<Rect>,
        dst: Rect,
        angle: Option<f32>,
        flip_x: bool,
        flip_y: bool,
    );
    fn create_texture(&mut self, handle: Handle, data: Vec<u8>, size: UVec2);
    fn remove_texture(&mut self, handle_id: HandleId);
    #[allow(async_fn_in_trait)]
    async fn run<Setup: FnOnce(&mut Engine, &mut World)>(
        title: String,
        width: u32,
        height: u32,
        vsync: bool,
        setup: Setup,
    ) -> Result<()>
    where
        Self: Sized;
}

pub(crate) async fn platform_run<Setup: FnOnce(&mut Engine, &mut World)>(
    title: String,
    width: u32,
    height: u32,
    vsync: bool,
    setup: Setup,
) -> anyhow::Result<()> {
    #[cfg(target_arch = "wasm32")]
    web::WebPlatform::run(title, width, height, vsync, setup).await?;
    #[cfg(not(target_arch = "wasm32"))]
    sdl::SDLPlatform::run(title, width, height, vsync, setup).await?;
    Ok(())
}
