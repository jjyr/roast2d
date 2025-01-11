use anyhow::{anyhow, bail};
use roast2d::map::Tile;
use roast2d::prelude::*;

use roast2d_physics::collision_map::{CollisionMap, DefaultCollisionRule, COLLISION_MAP};

use crate::{
    ldtk::{LayerType, LdtkLevel, LdtkLevelLayerInstance, LdtkProject},
    map::BackgroundMaps,
};

pub(crate) const FOREGROUND: &str = "foreground";
pub(crate) const DISTANCE: &str = "distance";
pub(crate) const DEFAULT_DISTANCE: f32 = 1.0;

pub fn build_collision_map_from_ldtk_layer(layer: &LdtkLevelLayerInstance) -> Result<CollisionMap> {
    if layer.r#type != LayerType::IntGrid {
        bail!("Collision map must be IntGrid type");
    }
    let size = UVec2::new(layer.c_wid, layer.c_hei);
    let tile_size = layer.grid_size as f32;
    let name = layer.identifier.clone();
    let data = layer.int_grid_csv.clone();

    let map = CollisionMap {
        name,
        size,
        tile_size,
        data,
        collision_rule: Box::new(DefaultCollisionRule),
    };
    log::debug!("Set Collision map {:?}", &map);
    Ok(map)
}

pub fn build_map_from_ldtk_layer(
    project: &LdtkProject,
    level: &LdtkLevel,
    layer_index: usize,
    layer: &LdtkLevelLayerInstance,
    tileset_texture: Handle,
) -> Result<Map> {
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

/// Load level
pub fn load_level<InitEntF: Fn(&mut World, &str, Transform, serde_json::Value) -> Result<Ent>>(
    g: &mut Engine,
    w: &mut World,
    proj: &LdtkProject,
    identifier: &str,
    init_ent_func: InitEntF,
) -> Result<()> {
    let level = proj.get_level(identifier)?;
    let mut background_maps = BackgroundMaps::default();
    g.input.clear();

    for (index, layer) in level.layer_instances.iter().enumerate() {
        match layer.r#type {
            LayerType::IntGrid if layer.identifier == COLLISION_MAP => {
                let map = build_collision_map_from_ldtk_layer(layer)?;
                w.add_resource(map);
            }
            LayerType::AutoLayer | LayerType::Tiles => {
                let tileset = if let Some(rel_path) = layer.tileset_rel_path.as_ref() {
                    g.assets.load_texture(rel_path)
                } else {
                    bail!(
                        "Layer {}-{} doesn't has tileset",
                        level.identifier,
                        &layer.identifier
                    )
                };
                let map = build_map_from_ldtk_layer(proj, level, index, layer, tileset)?;
                background_maps.add_background_map(map);
            }
            LayerType::Entities => {
                // spawn entities
                for ent_ins in &layer.entity_instances {
                    let pos = Vec2::new(
                        (ent_ins.px.0 + ent_ins.width / 2) as f32,
                        (ent_ins.px.1 + ent_ins.height / 2) as f32,
                    );
                    let identifier = &ent_ins.identifier;
                    let settings = ent_ins
                        .field_instances
                        .iter()
                        .map(|f| (f.identifier.clone(), f.value.clone()))
                        .collect();
                    let transform =
                        Transform::new(pos, Vec2::new(ent_ins.width as f32, ent_ins.height as f32));
                    init_ent_func(w, identifier, transform, settings)?;
                }
            }
            _ => {
                log::error!("Ignore layer {} {:?}", layer.identifier, layer.r#type);
            }
        }
    }

    Ok(())
}
