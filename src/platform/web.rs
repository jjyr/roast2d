use wasm_bindgen::prelude::wasm_bindgen;

use super::Platform;

pub struct WebPlatform;

impl Platform for WebPlatform {
    fn prepare_frame(&mut self) {
        todo!()
    }

    fn end_frame(&mut self) {
        todo!()
    }

    fn cleanup(&mut self) {
        todo!()
    }

    fn draw(
        &mut self,
        texture: &crate::handle::Handle,
        color: crate::prelude::Color,
        pos: glam::Vec2,
        size: glam::Vec2,
        uv_offset: glam::Vec2,
        uv_size: glam::Vec2,
        angle: f32,
        flip_x: bool,
        flip_y: bool,
    ) {
        todo!()
    }

    fn create_texture(&mut self, data: Vec<u8>, size: glam::UVec2) -> crate::handle::Handle {
        todo!()
    }

    fn run<Setup: FnOnce(&mut crate::prelude::Engine)>(
        title: String,
        width: u32,
        height: u32,
        vsync: bool,
        setup: Setup,
    ) -> anyhow::Result<()> {
        log::error!("Hello");
        alert("Hello from web backend");
        Ok(())
    }
}

#[wasm_bindgen]
extern "C" {
    fn alert(msg: &str);
}
