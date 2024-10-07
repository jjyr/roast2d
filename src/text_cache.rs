use std::collections::HashMap;

use roast2d_derive::Resource;

use crate::{
    engine::Engine,
    font::{Font, Text},
    prelude::World,
    sprite::Sprite,
};

/// Text cache
#[derive(Resource)]
pub struct TextCache {
    pub(crate) fonts: HashMap<u64, Font>,
    pub(crate) cache: HashMap<Text, Sprite>,
    pub(crate) max_text_cache: usize,
}

impl Default for TextCache {
    fn default() -> Self {
        Self {
            fonts: Default::default(),
            cache: Default::default(),
            max_text_cache: 1024,
        }
    }
}

impl TextCache {
    pub fn add(&mut self, text: Text, sprite: Sprite) {
        if self.cache.len() > self.max_text_cache {
            // randomly clean cache
            self.cache.retain(|_k, v| v.texture.id() & 1 == 0);
        }

        self.cache.insert(text, sprite);
    }

    pub fn get(&self, text: &Text) -> Option<&Sprite> {
        self.cache.get(text)
    }

    pub fn get_font(&mut self, handle_id: u64) -> Option<&Font> {
        self.fonts.get(&handle_id)
    }

    pub fn add_font(&mut self, handle_id: u64, font: Font) {
        self.fonts.insert(handle_id, font);
    }

    pub fn remove_font(&mut self, handle_id: u64) {
        self.fonts.remove(&handle_id);
    }
}

pub(crate) fn init_text_cache(_eng: &mut Engine, w: &mut World) {
    w.add_resource(TextCache::default());
}
