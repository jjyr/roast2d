use std::rc::Rc;

use roast2d_derive::Component;

use crate::{
    entity_hooks::EntHooks,
    prelude::{Ent, World},
};

#[derive(Component)]
pub struct Hooks {
    hooks: Rc<dyn EntHooks>,
}

impl Hooks {
    pub fn get(&self) -> Rc<dyn EntHooks> {
        self.hooks.clone()
    }
}

pub(crate) fn get_ent_hooks(w: &mut World, ent: Ent) -> Option<Rc<dyn EntHooks>> {
    w.get(ent)
        .and_then(|ent_ref| ent_ref.get::<Hooks>().map(|h| h.get()))
}
