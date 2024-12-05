use anyhow::bail;
use roast2d_derive::Resource;

use crate::{collision_map::COLLISION_MAP, entities::Commands, ldtk::*, prelude::*};

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

/// Load level
pub fn load_level(
    g: &mut Engine,
    w: &mut World,
    proj: &LdtkProject,
    identifier: &str,
) -> Result<()> {
    let level = proj.get_level(identifier)?;
    let mut background_maps = BackgroundMaps::default();
    g.input.clear();

    for (index, layer) in level.layer_instances.iter().enumerate() {
        match layer.r#type {
            LayerType::IntGrid if layer.identifier == COLLISION_MAP => {
                let map = CollisionMap::from_ldtk_layer(layer)?;
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
                let map = Map::from_ldtk_layer(proj, level, index, layer, tileset)?;
                background_maps.add_background_map(map);
            }
            LayerType::Entities => {
                // spawn entities
                for ent_ins in &layer.entity_instances {
                    let pos = Vec2::new(
                        (ent_ins.px.0 + ent_ins.width / 2) as f32,
                        (ent_ins.px.1 + ent_ins.height / 2) as f32,
                    );
                    let ent = {
                        let identifier = &ent_ins.identifier;
                        let mut ent = w.spawn();
                        // add transform
                        ent.add(Transform::new(
                            pos,
                            Vec2::new(ent_ins.width as f32, ent_ins.height as f32),
                        ))
                        // add same name component
                        .add_by_name(identifier);
                        ent.id()
                    };
                    let settings = ent_ins
                        .field_instances
                        .iter()
                        .map(|f| (f.identifier.clone(), f.value.clone()))
                        .collect();
                    w.get_resource_mut::<Commands>()?.setting(ent, settings);
                }
            }
            _ => {
                log::error!("Ignore layer {} {:?}", layer.identifier, layer.r#type);
            }
        }
    }

    Ok(())
}
