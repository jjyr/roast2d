use std::any::Any;

use anyhow::Result;
use glam::Vec2;

use crate::{
    engine::Engine,
    health::Health,
    prelude::{Ent, World},
    sprite::Sprite,
    trace::Trace,
    transform::Transform,
};

/// EntHooks
///
/// Use EntHooks to customize entity callback behaviors.
pub trait EntHooks {
    /// Load entity settings
    fn settings(
        &self,
        _eng: &mut Engine,
        _w: &mut World,
        _ent: Ent,
        _settings: serde_json::Value,
    ) -> Result<()> {
        Ok(())
    }

    /// Update callback is called before the entity_base_update
    fn update(&self, _eng: &mut Engine, _w: &mut World, _ent: Ent) -> Result<()> {
        Ok(())
    }

    /// Post update callback is called after the entity_base_update
    fn post_update(&self, _eng: &mut Engine, _w: &mut World, _ent: Ent) -> Result<()> {
        Ok(())
    }

    // Draw entity anim
    fn draw(&self, eng: &mut Engine, w: &mut World, ent: Ent, viewport: Vec2) -> Result<()> {
        let ent = w.get(ent)?;
        let sprite = ent.get::<Sprite>()?;
        let transform = ent.get::<Transform>()?;
        eng.render.borrow_mut().draw_image(
            sprite,
            transform.pos - viewport,
            Some(transform.scale),
            Some(transform.angle),
        );
        Ok(())
    }

    /// Called when entity is removed through kill
    fn kill(&self, _eng: &mut Engine, _w: &mut World, _ent: Ent) -> Result<()> {
        Ok(())
    }

    /// Called if one entity is touched by another entity
    fn touch(&self, _eng: &mut Engine, _w: &mut World, _ent: Ent, _other: Ent) -> Result<()> {
        Ok(())
    }

    /// Called when two entity are collide
    fn collide(
        &self,
        _eng: &mut Engine,
        _w: &mut World,
        _ent: Ent,
        _normal: Vec2,
        _trace: Option<&Trace>,
    ) -> Result<()> {
        Ok(())
    }

    /// Called when entity get damage
    fn damage(
        &self,
        eng: &mut Engine,
        w: &mut World,
        ent: Ent,
        _other: Ent,
        damage: f32,
    ) -> Result<()> {
        let mut ent = w.get_mut(ent)?;
        let health = ent.get_mut::<Health>()?;
        health.health -= damage;
        if health.health < 0.0 && health.alive {
            eng.kill(ent.id());
        }
        Ok(())
    }

    /// Called when entity is triggerred by another entity
    fn trigger(&self, _eng: &mut Engine, _w: &mut World, _ent: Ent, _other: Ent) -> Result<()> {
        Ok(())
    }

    /// Called when entity recives a message
    fn message(
        &self,
        _eng: &mut Engine,
        _w: &mut World,
        _ent: Ent,
        _data: Box<dyn Any>,
    ) -> Result<()> {
        Ok(())
    }
}
