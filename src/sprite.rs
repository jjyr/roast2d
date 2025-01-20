use core::fmt;
use std::fmt::Debug;

use glam::{UVec2, Vec2};
use roast2d_derive::Component;

use crate::{
    color::{Color, WHITE},
    handle::Handle,
    types::Rect,
};

impl Debug for Sprite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let size = self.size();
        f.debug_struct("Sprite").field("size", &size).finish()
    }
}

/// Sprite
#[derive(Clone, Component)]
pub struct Sprite {
    /// texture
    pub texture: Handle,
    /// src rect
    pub src: Option<Rect>,
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
    /// Anchor default (0.5, 0.5) is center
    pub anchor: Vec2,
}

impl Sprite {
    /// Build image from texture
    pub fn new(texture: Handle, size: UVec2) -> Self {
        Self {
            texture,
            size,
            src: None,
            color: WHITE,
            spacing: 0.0,
            padding: 0.0,
            flip_x: false,
            flip_y: false,
            anchor: Vec2::splat(0.5),
        }
    }

    /// Build image from texture
    pub fn with_sizef(texture: Handle, size: Vec2) -> Self {
        let size = UVec2::new(size.x as u32, size.y as u32);
        Self::new(texture, size)
    }

    /// Return image size
    pub fn size(&self) -> UVec2 {
        self.size
    }

    /// Return image size in Vec2
    pub fn sizef(&self) -> Vec2 {
        Vec2::new(self.size.x as f32, self.size.y as f32)
    }
}
