use std::rc::Rc;

use roast2d::{
    derive::{Component, Resource},
    prelude::*,
};

use crate::{collision, physics, trace::Trace};
use std::any::Any;

use anyhow::Result;
use glam::Vec2;
use serde_json::Value;

pub fn draw_entities(g: &mut Engine, w: &mut World) {
    let viewport = g.viewport();
    // Sort entities by draw_order
    let mut ents: Vec<_> = w
        .iter_ents_ref()
        .filter_map(|ent_ref| {
            let transform = ent_ref.get::<Transform>().ok()?;
            Some((ent_ref.id(), transform.z_index))
        })
        .collect();
    ents.sort_by_key(|(_ent, z)| *z);
    for (ent, _z) in ents {
        if let Ok(hooks) = get_ent_hooks(w, ent) {
            if let Err(err) = hooks.draw(g, w, ent, viewport) {
                log::error!("Error occured when call draw hooks on {ent:?}: {err}");
            }
        }
    }
}

pub fn update_entities(g: &mut Engine, w: &mut World) {
    // Update all entities
    let ents: Vec<_> = w.iter_ents().cloned().collect();
    for ent in ents {
        let ent_hooks = get_ent_hooks(w, ent);
        if let Ok(hooks) = ent_hooks.as_ref() {
            if let Err(err) = hooks.update(g, w, ent) {
                log::error!("Error when update {ent:?}: {err:?}");
            }
        }
        // physics update
        physics::entity_base_update(g, w, ent);
        if let Ok(hooks) = ent_hooks {
            if let Err(err) = hooks.post_update(g, w, ent) {
                log::error!("Error when update {ent:?}: {err:?}");
            }
        }
    }

    collision::update_collision(g, w);
    handle_commands(g, w);
}

/// Init commands
pub fn init_commands(_g: &mut Engine, w: &mut World) {
    w.add_resource(Commands::default());
}

/// Handle commands
pub(crate) fn handle_commands(g: &mut Engine, w: &mut World) {
    let Ok(queue) = w.get_resource_mut::<Commands>() else {
        return;
    };
    let commands = queue.take();
    for command in commands {
        if let Err(err) = handle_command(g, w, command) {
            log::debug!("Error occured when handling commands {err:?}");
        }
    }
}

/// Handle command
fn handle_command(g: &mut Engine, w: &mut World, command: Command) -> anyhow::Result<()> {
    match command {
        Command::Collide { ent, normal, trace } => {
            let hooks = get_ent_hooks(w, ent)?;
            hooks.collide(g, w, ent, normal, trace.as_ref())?;
        }
        Command::Setting { ent, settings } => {
            let hooks = get_ent_hooks(w, ent)?;
            hooks.settings(g, w, ent, settings)?;
        }
        Command::KillEnt { ent } => {
            let mut ent_ref = w.get_mut(ent)?;
            if let Ok(health) = ent_ref.get_mut::<Health>() {
                health.killed = true;
            }
            let hooks = get_ent_hooks(w, ent)?;
            hooks.kill(g, w, ent)?;
            w.despawn(ent);
        }
        Command::Damage {
            ent,
            by_ent,
            damage,
        } => {
            let hooks = get_ent_hooks(w, ent)?;
            hooks.damage(g, w, ent, by_ent, damage)?;
        }
        Command::Trigger { ent, other } => {
            let hooks = get_ent_hooks(w, ent)?;
            hooks.trigger(g, w, ent, other)?;
        }
        Command::Message { ent, data } => {
            let hooks = get_ent_hooks(w, ent)?;
            hooks.message(g, w, ent, data)?;
        }
    }
    Ok(())
}

#[derive(Default, Resource)]
pub struct Commands {
    buffer: Vec<Command>,
}

impl Commands {
    pub(crate) fn add(&mut self, cmd: Command) {
        self.buffer.push(cmd);
    }

    pub(crate) fn take(&mut self) -> Vec<Command> {
        std::mem::take(&mut self.buffer)
    }

    pub(crate) fn collide(&mut self, ent: Ent, normal: Vec2, trace: Option<Trace>) {
        self.add(Command::Collide { ent, normal, trace });
    }

    /// Setting an entity
    pub fn setting(&mut self, ent: Ent, settings: serde_json::Value) {
        self.add(Command::Setting { ent, settings });
    }

    /// Kill an entity
    pub fn kill(&mut self, ent: Ent) {
        self.add(Command::KillEnt { ent });
    }

    /// Damage an entity
    pub fn damage(&mut self, ent: Ent, by_ent: Ent, damage: f32) {
        self.add(Command::Damage {
            ent,
            by_ent,
            damage,
        });
    }

    /// Trigger an entity
    pub fn trigger(&mut self, ent: Ent, other: Ent) {
        self.add(Command::Trigger { ent, other });
    }

    /// Message an entity
    pub fn message(&mut self, ent: Ent, data: Box<dyn Any>) {
        self.add(Command::Message { ent, data });
    }
}

pub(crate) enum Command {
    Collide {
        ent: Ent,
        normal: Vec2,
        trace: Option<Trace>,
    },
    Setting {
        ent: Ent,
        settings: Value,
    },
    KillEnt {
        ent: Ent,
    },
    Damage {
        ent: Ent,
        by_ent: Ent,
        damage: f32,
    },
    Trigger {
        ent: Ent,
        other: Ent,
    },
    Message {
        ent: Ent,
        data: Box<dyn Any>,
    },
}

/// EntHooks
///
/// Use EntHooks to customize entity callback behaviors.
pub trait EntHooks {
    /// Load entity settings
    fn settings(
        &self,
        _g: &mut Engine,
        _w: &mut World,
        _ent: Ent,
        _settings: serde_json::Value,
    ) -> Result<()> {
        Ok(())
    }

    /// Update callback is called before the entity_base_update
    fn update(&self, _g: &mut Engine, _w: &mut World, _ent: Ent) -> Result<()> {
        Ok(())
    }

    /// Post update callback is called after the entity_base_update
    fn post_update(&self, _g: &mut Engine, _w: &mut World, _ent: Ent) -> Result<()> {
        Ok(())
    }

    // Draw entity anim
    fn draw(&self, g: &mut Engine, w: &mut World, ent: Ent, viewport: Vec2) -> Result<()> {
        let ent = w.get(ent)?;
        let sprite = ent.get::<Sprite>()?;
        let transform = ent.get::<Transform>()?;
        g.draw_image(
            sprite,
            transform.pos - viewport,
            Some(transform.scale),
            Some(transform.angle),
        );
        Ok(())
    }

    /// Called when entity is removed through kill
    fn kill(&self, _g: &mut Engine, _w: &mut World, _ent: Ent) -> Result<()> {
        Ok(())
    }

    /// Called if one entity is touched by another entity
    fn touch(&self, _g: &mut Engine, _w: &mut World, _ent: Ent, _other: Ent) -> Result<()> {
        Ok(())
    }

    /// Called when two entity are collide
    fn collide(
        &self,
        _g: &mut Engine,
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
        _g: &mut Engine,
        w: &mut World,
        ent: Ent,
        _other: Ent,
        damage: f32,
    ) -> Result<()> {
        let mut ent = w.get_mut(ent)?;
        let health = ent.get_mut::<Health>()?;
        health.value -= damage;
        if !health.is_alive() && !health.killed {
            let id = ent.id();
            w.get_resource_mut::<Commands>()?.kill(id);
        }
        Ok(())
    }

    /// Called when entity is triggerred by another entity
    fn trigger(&self, _g: &mut Engine, _w: &mut World, _ent: Ent, _other: Ent) -> Result<()> {
        Ok(())
    }

    /// Called when entity recives a message
    fn message(
        &self,
        _g: &mut Engine,
        _w: &mut World,
        _ent: Ent,
        _data: Box<dyn Any>,
    ) -> Result<()> {
        Ok(())
    }
}

#[derive(Component)]
pub struct Hooks {
    hooks: Rc<dyn EntHooks>,
}

impl Hooks {
    pub fn new<T: EntHooks + 'static>(ent_hooks: T) -> Self {
        Self {
            hooks: Rc::new(ent_hooks),
        }
    }
    pub fn get(&self) -> Rc<dyn EntHooks> {
        self.hooks.clone()
    }
}

pub(crate) fn get_ent_hooks(w: &mut World, ent: Ent) -> Result<Rc<dyn EntHooks>, Error> {
    w.get(ent)
        .and_then(|ent_ref| ent_ref.get::<Hooks>().map(|h| h.get()))
}
