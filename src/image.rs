use core::fmt;
use std::fmt::Debug;

use glam::{UVec2, Vec2};
use sdl2::render::Texture;

use crate::{
    color::{Color, WHITE},
    render::Render,
    types::Mut,
};

/// Image
/// Use Engine#load_image to get a image
#[derive(Clone)]
pub struct Image {
    /// texture
    pub texture: Mut<Texture>,
    /// image scale size
    pub scale: Vec2,
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

impl Image {
    /// Build image from texture
    pub fn new(texture: Texture) -> Self {
        Self {
            texture: Mut::new(texture),
            scale: Vec2::splat(1.0),
            color: WHITE,
            spacing: 0.0,
            padding: 0.0,
            flip_x: false,
            flip_y: false,
        }
    }
}

impl Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let size = self.size();
        f.debug_struct("Image").field("size", &size).finish()
    }
}

impl Image {
    /// Return image size
    pub fn size(&self) -> UVec2 {
        let query = self.texture.borrow().query();
        UVec2::new(query.width, query.height)
    }

    /// Return image size in Vec2
    pub fn sizef(&self) -> Vec2 {
        let query = self.texture.borrow().query();
        Vec2::new(query.width as f32, query.height as f32)
    }

    pub(crate) fn draw_tile_ex(
        &self,
        render: &mut Render,
        tile: u16,
        tile_size: Vec2,
        dst_pos: Vec2,
        flip_x: bool,
        flip_y: bool,
    ) {
        let cols =
            ((self.size().x as f32 - self.padding) / (tile_size.x + self.spacing)).round() as u32;
        let row = tile as u32 / cols;
        let col = tile as u32 % cols;
        let src_pos = Vec2::new(
            col as f32 * (tile_size.x + self.spacing) + self.padding,
            row as f32 * (tile_size.y + self.spacing) + self.padding,
        );
        let src_size = Vec2::new(tile_size.x, tile_size.y);
        let dst_size = src_size * self.scale;

        // color
        let mut texture = self.texture.borrow_mut();
        texture.set_color_mod(self.color.r, self.color.g, self.color.b);
        render.render_draw(
            dst_pos,
            dst_size,
            &texture,
            src_pos,
            src_size,
            flip_x || self.flip_x,
            flip_y || self.flip_y,
        );
    }
}
