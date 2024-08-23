use glam::Vec2;

use crate::{render::Render, sprite::Sprite};

#[derive(Clone)]
pub struct Animation {
    pub sheet: Sprite,
}

impl Animation {
    pub fn new(sheet: Sprite) -> Self {
        Self { sheet }
    }
    pub(crate) fn draw(&mut self, render: &mut Render, pos: Vec2) {
        let size = self.sheet.size();
        let size = Vec2::new(size.x as f32, size.y as f32);
        render.draw_tile(&self.sheet, 0, size, pos, false, false)
    }
}
