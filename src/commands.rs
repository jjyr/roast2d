use std::any::Any;

use glam::Vec2;
use serde_json::Value;

use crate::{entity::EntRef, trace::Trace};

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
        ent: EntRef,
        normal: Vec2,
        trace: Option<Trace>,
    },
    Setting {
        ent: EntRef,
        settings: Value,
    },
    KillEnt {
        ent: EntRef,
    },
    Damage {
        ent: EntRef,
        by_ent: EntRef,
        damage: f32,
    },
    Trigger {
        ent: EntRef,
        other: EntRef,
    },
    Message {
        ent: EntRef,
        data: Box<dyn Any>,
    },
}
