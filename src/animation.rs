use glam::Vec2;

use crate::{image::Image, render::Render};

#[derive(Clone)]
pub struct Animation {
    pub sheet: Image,
}

impl Animation {
    pub fn new(sheet: Image) -> Self {
        Self { sheet }
    }
    pub(crate) fn draw(&mut self, render: &mut Render, pos: Vec2) {
        let size = self.sheet.size();
        let size = Vec2::new(size.x as f32, size.y as f32);
        self.sheet.draw_tile_ex(render, 0, size, pos, false, false);
    }
}
