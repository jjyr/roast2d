use std::any::Any;

use glam::Vec2;
use serde_json::Value;

use crate::{ecs::entity::Ent, trace::Trace};

#[derive(Default)]
pub(crate) struct Commands {
    buffer: Vec<Command>,
}

impl Commands {
    pub(crate) fn add(&mut self, cmd: Command) {
        self.buffer.push(cmd);
    }

    pub(crate) fn take(&mut self) -> Vec<Command> {
        std::mem::take(&mut self.buffer)
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
