use anyhow::{anyhow, Result};
use glam::{UVec2, Vec2};

use crate::{
    handle::Handle,
    ldtk::{LdtkLevel, LdtkLevelLayerInstance, LdtkProject},
    render::Render,
    sprite::Sprite,
};

pub(crate) const FOREGROUND: &str = "foreground";
pub(crate) const DISTANCE: &str = "distance";
pub(crate) const DEFAULT_DISTANCE: f32 = 1.0;

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

    pub(crate) fn from_ldtk_layer(
        project: &LdtkProject,
        level: &LdtkLevel,
        layer_index: usize,
        layer: &LdtkLevelLayerInstance,
        tileset_texture: Handle,
    ) -> Result<Self> {
        let tileset_def = layer
            .tileset_def_uid
            .and_then(|uid| project.get_tileset(uid))
            .ok_or_else(|| anyhow!("No tileset def"))?;
        let mut tileset = {
            let size = UVec2::new(tileset_def.px_wid, tileset_def.px_hei);
            Sprite::new(tileset_texture, size)
        };
        tileset.spacing = tileset_def.spacing as f32;
        tileset.padding = tileset_def.padding as f32;

        let size = UVec2::new(layer.c_wid, layer.c_hei);
        let tile_size = layer.grid_size as f32;
        let name = layer.identifier.clone();
        let distance: f32 = level.get_nth(DISTANCE, layer_index, DEFAULT_DISTANCE)?;
        let foreground: bool = level.get_nth(FOREGROUND, layer_index, false)?;
        let mut data = Vec::with_capacity(layer.auto_layer_tiles.len() + layer.grid_tiles.len());

        // Read auto layer tiles
        let auto_tiles = layer.auto_layer_tiles.iter().map(|auto_tile| {
            let flip_x = auto_tile.x_flip();
            let flip_y = auto_tile.y_flip();
            let tile_id = auto_tile.t;
            let dst = Vec2::new(auto_tile.px.0 as f32, auto_tile.px.1 as f32);
            Tile {
                flip_x,
                flip_y,
                tile_id,
                dst,
            }
        });
        data.extend(auto_tiles);

        // Read tiles
        let tiles = layer.grid_tiles.iter().map(|auto_tile| {
            let flip_x = auto_tile.x_flip();
            let flip_y = auto_tile.y_flip();
            let tile_id = auto_tile.t;
            let dst = Vec2::new(auto_tile.px.0 as f32, auto_tile.px.1 as f32);
            Tile {
                flip_x,
                flip_y,
                tile_id,
                dst,
            }
        });
        data.extend(tiles);

        let map = Map {
            name,
            size,
            tile_size,
            distance,
            foreground,
            tileset,
            data,
        };
        Ok(map)
    }
}

fn map_draw_tile(render: &mut Render, map: &Map, tile: &Tile, pos: Vec2) {
    render.draw_tile(
        &map.tileset,
        tile.tile_id,
        Vec2::splat(map.tile_size),
        pos,
        None,
        None,
        tile.flip_x,
        tile.flip_y,
    );
}

pub(crate) fn map_draw(render: &mut Render, map: &Map, mut offset: Vec2) {
    offset /= map.distance;
    let half_tile_size = map.tile_size * 0.5;

    for tile in map.data.iter() {
        let pos = tile.dst - offset + half_tile_size;
        map_draw_tile(render, map, tile, pos);
    }
}
