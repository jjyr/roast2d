use anyhow::{anyhow, Result};
use de::DeserializeOwned;
use glam::Vec2;
use serde::*;
use serde_json::Value;

use crate::types::Rect;

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LdtkProject {
    pub json_version: String,
    pub defs: LdtkDefs,
    pub levels: Vec<LdtkLevel>,
}

impl LdtkProject {
    pub fn get_tileset(&self, uid: u32) -> Option<&LdtkTileset> {
        self.defs.tilesets.iter().find(|t| t.uid == uid)
    }

    pub fn get_entity(&self, uid: u32) -> Option<&LdtkEntity> {
        self.defs.entities.iter().find(|e| e.uid == uid)
    }

    pub fn get_entity_by_name(&self, identifier: &str) -> Option<&LdtkEntity> {
        self.defs
            .entities
            .iter()
            .find(|e| e.identifier == identifier)
    }

    pub fn get_enum_by_name(&self, identifier: &str) -> Option<&LdtkEnum> {
        self.defs.enums.iter().find(|e| e.identifier == identifier)
    }

    pub fn get_level(&self, identifier: &str) -> Result<&LdtkLevel> {
        self.levels
            .iter()
            .find(|l| l.identifier == identifier)
            .ok_or_else(|| anyhow!("can't find level"))
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LdtkDefs {
    pub layers: Vec<LdtkLayer>,
    pub entities: Vec<LdtkEntity>,
    pub tilesets: Vec<LdtkTileset>,
    pub enums: Vec<LdtkEnum>,
    pub level_fields: Vec<LdtkLevelField>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LayerType {
    IntGrid,
    Entities,
    Tiles,
    AutoLayer,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LdtkLayer {
    pub identifier: String,
    pub r#type: LayerType,
    pub uid: u32,
    pub grid_size: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LdtkEntity {
    pub identifier: String,
    pub uid: u32,
    pub width: usize,
    pub height: usize,
    pub color: String,
    pub tileset_id: Option<u32>,
    pub tile_rect: Option<LdtkTileRect>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LdtkTileRect {
    pub tileset_uid: u32,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl From<LdtkTileRect> for Rect {
    fn from(val: LdtkTileRect) -> Self {
        let min = Vec2::new(val.x as f32, val.y as f32);
        let max = Vec2::new((val.x + val.w) as f32, (val.y + val.h) as f32);
        Rect { min, max }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LdtkTileset {
    pub identifier: String,
    pub uid: u32,
    pub rel_path: Option<String>,
    pub px_wid: u32,
    pub px_hei: u32,
    pub tile_grid_size: u32,
    pub tag_source_enum_uid: Option<u32>,
    pub spacing: u32,
    pub padding: u32,
}

impl LdtkTileset {
    pub fn columns(&self) -> u32 {
        self.px_wid / self.tile_grid_size
    }

    pub fn rows(&self) -> u32 {
        self.px_hei / self.tile_grid_size
    }

    pub fn tile_id(&self, tile: &LdtkTileRect) -> u32 {
        let x = tile.x / self.tile_grid_size;
        let y = tile.y / self.tile_grid_size;
        y * self.columns() + x
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LdtkEnum {
    pub identifier: String,
    pub uid: u32,
    pub values: Vec<LdtkEnumValue>,
}

impl LdtkEnum {
    pub fn get_value(&self, id: &str) -> Option<&LdtkEnumValue> {
        self.values.iter().find(|v| v.id == id)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LdtkEnumValue {
    pub id: String,
    pub tile_rect: Option<LdtkTileRect>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LdtkLevelField {
    pub identifier: String,
    pub uid: u32,
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LdtkLevel {
    pub identifier: String,
    pub iid: String,
    pub uid: u32,
    pub bg_rel_path: Option<String>,
    pub bg_color: Option<String>,
    pub field_instances: Vec<LdtkFieldInstance>,
    pub layer_instances: Vec<LdtkLevelLayerInstance>,
}

impl LdtkLevel {
    pub fn get_field(&self, identifier: &str) -> Option<&LdtkFieldInstance> {
        self.field_instances
            .iter()
            .find(|f| f.identifier == identifier)
    }

    pub fn get<T: DeserializeOwned>(&self, identifier: &str, default: T) -> Result<T> {
        if let Some(f) = self.get_field(identifier) {
            serde_json::from_value(f.value.clone()).map_err(Into::into)
        } else {
            Ok(default)
        }
    }

    pub fn get_nth<T: DeserializeOwned + Clone>(
        &self,
        identifier: &str,
        index: usize,
        default: T,
    ) -> Result<T> {
        let list: Vec<T> = self.get(identifier, Vec::default())?;
        let elem = list.get(index).cloned().unwrap_or(default);
        Ok(elem)
    }

    pub fn get_value(&self, identifier: &str) -> Option<&Value> {
        self.get_field(identifier).map(|f| &f.value)
    }

    pub fn get_layer(&self, identifier: &str) -> Option<&LdtkLevelLayerInstance> {
        self.layer_instances
            .iter()
            .find(|l| l.identifier == identifier)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LdtkFieldInstance {
    #[serde(rename = "__identifier")]
    pub identifier: String,
    #[serde(rename = "__type")]
    pub r#type: String,
    #[serde(rename = "__value")]
    pub value: serde_json::Value,
    pub def_uid: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LdtkLevelLayerInstance {
    #[serde(rename = "__identifier")]
    pub identifier: String,
    #[serde(rename = "__type")]
    pub r#type: LayerType,
    #[serde(rename = "__cWid")]
    pub c_wid: u32,
    #[serde(rename = "__cHei")]
    pub c_hei: u32,
    #[serde(rename = "__gridSize")]
    pub grid_size: u32,
    #[serde(rename = "__tilesetDefUid")]
    pub tileset_def_uid: Option<u32>,
    #[serde(rename = "__tilesetRelPath")]
    pub tileset_rel_path: Option<String>,
    pub iid: String,
    pub level_id: usize,
    pub layer_def_uid: usize,
    pub entity_instances: Vec<LdtkEntityInstance>,
    pub auto_layer_tiles: Vec<LdtkTile>,
    pub grid_tiles: Vec<LdtkTile>,
    pub int_grid_csv: Vec<u16>,
}

pub type LdtkGrid = (u32, u32);
pub type LdtkGridF = (f32, f32);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LdtkEntityInstance {
    #[serde(rename = "__identifier")]
    pub identifier: String,
    pub iid: String,
    #[serde(rename = "__grid")]
    pub grid: LdtkGrid,
    #[serde(rename = "__pivot")]
    pub pivot: LdtkGridF,
    #[serde(rename = "__tile")]
    pub tile: Option<LdtkTileRect>,
    pub width: u32,
    pub height: u32,
    pub def_uid: usize,
    pub px: LdtkGrid,
    pub field_instances: Vec<LdtkFieldInstance>,
}

impl LdtkEntityInstance {
    pub fn get_field(&self, identifier: &str) -> Option<&LdtkFieldInstance> {
        self.field_instances
            .iter()
            .find(|f| f.identifier == identifier)
    }

    pub fn get(&self, identifier: &str) -> Option<&Value> {
        self.get_field(identifier).map(|f| &f.value)
    }

    pub fn get_bool(&self, identifier: &str, default: bool) -> bool {
        self.get_field(identifier)
            .map(|f| &f.value)
            .and_then(|v| v.as_bool())
            .unwrap_or(default)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LdtkTile {
    pub px: LdtkGrid,
    pub src: LdtkGrid,
    pub f: u8,
    pub t: u16,
    pub d: Vec<u16>,
    pub a: u16,
}

impl LdtkTile {
    pub fn x_flip(&self) -> bool {
        (self.f & 0b01) != 0
    }

    pub fn y_flip(&self) -> bool {
        (self.f & 0b10) != 0
    }
}
