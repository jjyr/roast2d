use roast2d::{derive::Resource, map::Tile, prelude::*};

// The maps to draw. Reset for each scene. Use engine_add_background_map()
// to add.
#[derive(Resource, Default)]
pub struct BackgroundMaps {
    pub maps: Vec<Map>,
}

impl BackgroundMaps {
    /// Add background map
    pub fn add_background_map(&mut self, map: Map) {
        self.maps.push(map);
    }
}

pub fn draw_maps(g: &mut Engine, w: &mut World, foreground: bool) {
    let viewport = g.viewport();

    if let Ok(background_maps) = w.get_resource::<BackgroundMaps>() {
        for map in background_maps.maps.iter().rev() {
            if map.foreground == foreground {
                draw_map_tiles(g, map, viewport);
            }
        }
    }
}

fn draw_tile(g: &mut Engine, map: &Map, tile: &Tile, pos: Vec2) {
    g.draw_tile(
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

pub(crate) fn draw_map_tiles(g: &mut Engine, map: &Map, mut offset: Vec2) {
    offset /= map.distance;
    let half_tile_size = map.tile_size * 0.5;

    for tile in map.data.iter() {
        let pos = tile.dst - offset + half_tile_size;
        draw_tile(g, map, tile, pos);
    }
}
