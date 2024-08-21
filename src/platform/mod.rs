use anyhow::Result;
use glam::{UVec2, Vec2};

use crate::{color::Color, engine::Engine, handle::Handle};

pub type DefaultPlatform = sdl::PlatformSDL;
mod sdl;

pub trait Platform {
    fn prepare_frame(&mut self);
    fn end_frame(&mut self);
    fn cleanup(&mut self);
    fn draw(
        &mut self,
        texture: &Handle,
        color: Color,
        pos: Vec2,
        size: Vec2,
        uv_offset: Vec2,
        uv_size: Vec2,
        angle: f32,
        flip_x: bool,
        flip_y: bool,
    );
    fn create_texture(&mut self, data: Vec<u8>, size: UVec2) -> Handle;
    fn run<Setup: FnOnce(&mut Engine)>(
        title: String,
        width: u32,
        height: u32,
        vsync: bool,
        setup: Setup,
    ) -> Result<()>
    where
        Self: Sized;
}
