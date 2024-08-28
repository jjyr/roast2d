use core::fmt;
use std::fmt::Debug;

use glam::{UVec2, Vec2};

use crate::{
    color::{Color, WHITE},
    handle::Handle,
};

/// Sprite
#[derive(Clone)]
pub struct Sprite {
    /// texture
    pub texture: Handle,
    /// size
    pub size: UVec2,
    /// color
    pub color: Color,
    /// Spacing
    pub spacing: f32,
    /// Padding
    pub padding: f32,
    /// Flip Horizontal
    pub flip_x: bool,
    /// Flip Vertical
    pub flip_y: bool,
}

impl Sprite {
    /// Build image from texture
    pub fn new(texture: Handle, size: UVec2) -> Self {
        Self {
            texture,
            size,
            color: WHITE,
            spacing: 0.0,
            padding: 0.0,
            flip_x: false,
            flip_y: false,
        }
    }

    /// Build image from texture
    pub fn with_sizef(texture: Handle, size: Vec2) -> Self {
        let size = UVec2::new(size.x as u32, size.y as u32);
        Self::new(texture, size)
    }
}

impl Debug for Sprite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let size = self.size();
        f.debug_struct("Image").field("size", &size).finish()
    }
}

impl Sprite {
    /// Return image size
    pub fn size(&self) -> UVec2 {
        self.size
    }

    /// Return image size in Vec2
    pub fn sizef(&self) -> Vec2 {
        Vec2::new(self.size.x as f32, self.size.y as f32)
    }
}
