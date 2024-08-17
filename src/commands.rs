use std::any::Any;

use serde_json::Value;

use crate::entity::EntityRef;

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
    Setting {
        ent: EntityRef,
        settings: Value,
    },
    KillEntity {
        ent: EntityRef,
    },
    Damage {
        ent: EntityRef,
        by_ent: EntityRef,
        damage: f32,
    },
    Trigger {
        ent: EntityRef,
        other: EntityRef,
    },
    Message {
        ent: EntityRef,
        msg_id: u32,
        data: Box<dyn Any>,
    },
}
