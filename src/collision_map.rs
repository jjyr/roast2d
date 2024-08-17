use anyhow::{bail, Result};
use glam::{IVec2, UVec2, Vec2};

use crate::ldtk::{LayerType, LdtkLevelLayerInstance};

pub(crate) const COLLISION_MAP: &str = "Collision";

/// Game map
#[derive(Debug, Default)]
pub struct CollisionMap {
    pub name: String,
    pub size: UVec2,
    pub tile_size: f32,
    // The tile indices with a length of size.x * size.y
    pub data: Vec<u16>,
}

impl CollisionMap {
    pub fn get(&self, pos: IVec2) -> Option<u16> {
        if pos.x < 0 || pos.y < 0 || pos.x >= self.size.x as i32 || pos.y >= self.size.y as i32 {
            return None;
        }
        let index = (pos.y * self.size.x as i32 + pos.x) as usize;
        self.data.get(index).cloned()
    }

    pub fn bounds(&self) -> Vec2 {
        Vec2::new(
            self.tile_size * self.size.x as f32,
            self.tile_size * self.size.y as f32,
        )
    }

    pub(crate) fn from_ldtk_layer(layer: &LdtkLevelLayerInstance) -> Result<Self> {
        if layer.r#type != LayerType::IntGrid {
            bail!("Collision map must be IntGrid type");
        }
        let size = UVec2::new(layer.c_wid, layer.c_hei);
        let tile_size = layer.grid_size as f32;
        let name = layer.identifier.clone();
        let data = layer.int_grid_csv.clone();

        let map = Self {
            name,
            size,
            tile_size,
            data,
        };
        Ok(map)
    }
}
