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
        render.draw_image(&self.sheet, pos)
    }
}
