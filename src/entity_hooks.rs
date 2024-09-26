use std::any::Any;

use glam::Vec2;

use crate::{
    engine::Engine,
    prelude::{Ent, World},
    sprite::Sprite,
    trace::Trace,
    transform::Transform,
};

pub trait EntHooks {
    /// Load entity settings
    fn settings(&self, _eng: &mut Engine, _w: &mut World, _ent: Ent, _settings: serde_json::Value) {
    }

    /// Update callback is called before the entity_base_update
    fn update(&self, _eng: &mut Engine, _w: &mut World, _ent: Ent) {}

    /// Post update callback is called after the entity_base_update
    fn post_update(&self, _eng: &mut Engine, _w: &mut World, _ent: Ent) {}

    // Draw entity anim
    fn draw(&self, eng: &mut Engine, w: &mut World, ent: Ent, viewport: Vec2) {
        let Some(ent) = w.get(ent) else {
            return;
        };
        let Some(sprite) = ent.get::<Sprite>() else {
            return;
        };
        let Some(transform) = ent.get::<Transform>() else {
            return;
        };
        eng.render.borrow_mut().draw_image(
            sprite,
            transform.pos - viewport,
            Some(transform.scale),
            Some(transform.angle),
        );
    }

    /// Called when entity is removed through kill
    fn kill(&self, _eng: &mut Engine, _w: &mut World, _ent: Ent) {}

    // /// Called if one entity is touched by another entity
    // fn touch(&mut self, _eng: &mut Engine, _w: &mut World, _ent: Ent, _other: Ent) {}

    /// Called when two entity are collide
    fn collide(
        &self,
        _eng: &mut Engine,
        _w: &mut World,
        _ent: Ent,
        _normal: Vec2,
        _trace: Option<&Trace>,
    ) {
    }

    /// Called when entity get damage
    fn damage(&self, eng: &mut Engine, w: &mut World, ent: Ent, _other: Ent, damage: f32) {
        // let Some(ent) = w.get_mut(ent) else {
        //     return;
        // };
        // ent.health -= damage;
        // if ent.health < 0.0 && ent.alive {
        //     let ent_ref = ent.ent_ref;
        //     eng.kill(ent_ref);
        // }
    }

    /// Called when entity is triggerred by another entity
    fn trigger(&self, _eng: &mut Engine, _w: &mut World, _ent: Ent, _other: Ent) {}

    /// Called when entity recives a message
    fn message(&self, _eng: &mut Engine, _w: &mut World, _ent: Ent, _data: Box<dyn Any>) {}
}
