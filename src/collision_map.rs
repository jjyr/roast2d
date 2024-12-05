use std::fmt::Debug;

use anyhow::{bail, Result};
use glam::{IVec2, UVec2, Vec2};
use roast2d_derive::Resource;

use crate::ldtk::{LayerType, LdtkLevelLayerInstance};

pub(crate) const COLLISION_MAP: &str = "Collision";

pub trait CollisionRule {
    fn is_collide(&self, map: &CollisionMap, pos: IVec2) -> bool;
}

#[derive(Default)]
pub struct DefaultCollisionRule;

impl CollisionRule for DefaultCollisionRule {
    fn is_collide(&self, map: &CollisionMap, pos: IVec2) -> bool {
        map.get(pos).is_some_and(|t| t != 0)
    }
}

/// Game map
#[derive(Resource)]
pub struct CollisionMap {
    pub name: String,
    pub size: UVec2,
    pub tile_size: f32,
    // The tile indices with a length of size.x * size.y
    pub data: Vec<u16>,
    pub collision_rule: Box<dyn CollisionRule>,
}

impl Debug for CollisionMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CollisionMap")
            .field("name", &self.name)
            .field("size", &self.size)
            .field("tile_size", &self.tile_size)
            .finish()
    }
}

impl Default for CollisionMap {
    fn default() -> Self {
        Self {
            name: "Collision".to_string(),
            size: UVec2::default(),
            tile_size: 0.0,
            data: Default::default(),
            collision_rule: Box::new(DefaultCollisionRule),
        }
    }
}

impl CollisionMap {
    pub fn get(&self, pos: IVec2) -> Option<u16> {
        if pos.x < 0 || pos.y < 0 || pos.x >= self.size.x as i32 || pos.y >= self.size.y as i32 {
            return None;
        }
        let index = (pos.y * self.size.x as i32 + pos.x) as usize;
        self.data.get(index).cloned()
    }

    pub fn set_collision_rule<CR: CollisionRule + 'static>(&mut self, rule: CR) {
        self.collision_rule = Box::new(rule);
    }

    pub fn is_collide(&self, pos: IVec2) -> bool {
        self.collision_rule.is_collide(self, pos)
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
            collision_rule: Box::new(DefaultCollisionRule),
        };
        log::debug!("Set Collision map {:?}", &map);
        Ok(map)
    }
}
