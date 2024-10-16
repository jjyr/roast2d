use glam::Vec2;
use roast2d_derive::Component;

use crate::{collision::calc_bounds, types::Rect};

#[derive(Component)]
pub struct Transform {
    pub pos: Vec2,
    pub scale: Vec2,
    /// Angle in radians
    pub angle: f32,
    pub size: Vec2,
    pub z_index: u32,
}

impl Transform {
    pub fn new(pos: Vec2, size: Vec2) -> Self {
        Self {
            pos,
            size,
            z_index: 0,
            scale: Vec2::splat(1.0),
            angle: 0.0,
        }
    }

    pub fn with_z_index(mut self, z_index: u32) -> Self {
        self.z_index = z_index;
        self
    }

    pub fn scaled_size(&self) -> Vec2 {
        self.size * self.scale
    }

    pub fn bounds(&self) -> Rect {
        let half_size = self.scaled_size() * 0.5;
        calc_bounds(self.pos, half_size, self.angle)
    }
}
