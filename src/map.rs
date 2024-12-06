use glam::{UVec2, Vec2};

use crate::sprite::Sprite;

#[derive(Debug, Clone)]
pub struct Tile {
    pub flip_x: bool,
    pub flip_y: bool,
    pub tile_id: u16,
    pub dst: Vec2,
}

/// Game map
#[derive(Debug)]
pub struct Map {
    pub name: String,
    pub size: UVec2,
    pub tile_size: f32,
    // The "distance" of the map when drawing at a certain offset. Maps that
    // have a higher distance move slower. Default 1.
    pub distance: f32,

    // Whether to draw this map in fround of all entities
    pub foreground: bool,

    // The tileset image to use when drawing. Might be NULL for collision maps
    pub tileset: Sprite,

    // The tile indices with a length of size.x * size.y
    pub data: Vec<Tile>,
}

impl Map {
    pub fn bounds(&self) -> Vec2 {
        Vec2::new(
            self.tile_size * self.size.x as f32,
            self.tile_size * self.size.y as f32,
        )
    }
}
