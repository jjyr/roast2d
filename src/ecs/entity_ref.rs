use std::any::type_name;

use super::{
    component::{Component, ComponentId},
    entity::Ent,
    unsafe_world_ref::UnsafeWorldRef,
};

/// Entity ref
/// Support access components of an entity
pub struct EntRef<'w> {
    ent: Ent,
    world_ref: UnsafeWorldRef<'w>,
}

impl<'w> EntRef<'w> {
    pub(crate) fn new(ent: Ent, world_ref: UnsafeWorldRef<'w>) -> EntRef<'w> {
        Self { ent, world_ref }
    }

    pub fn id(&self) -> Ent {
        self.ent
    }

    pub fn get<T: Component + 'static>(&'w self) -> Option<&'w T> {
        let w = unsafe { self.world_ref.as_ref() };
        w.storage
            .get(&ComponentId::of::<T>())?
            .get(&self.ent)
            .and_then(|b| b.as_any().downcast_ref())
    }
}

/// Entity ref
/// Support access components of an entity
pub struct EntMut<'w> {
    ent: Ent,
    world_ref: UnsafeWorldRef<'w>,
}

impl<'w> EntMut<'w> {
    pub(crate) fn new(ent: Ent, world_ref: UnsafeWorldRef<'w>) -> EntMut<'w> {
        Self { ent, world_ref }
    }

    pub fn id(&self) -> Ent {
        self.ent
    }

    pub fn add<T: Component + 'static>(&mut self, component: T) -> &mut Self {
        let w = unsafe { self.world_ref.as_mut() };
        if w.storage
            .entry(ComponentId::of::<T>())
            .or_default()
            .insert(self.ent, Box::new(component))
            .is_some()
        {
            panic!("Existed component {} {:?}", type_name::<T>(), self.ent);
        }
        self
    }

    pub fn get<T: Component + 'static>(&self) -> Option<&T> {
        let w = unsafe { self.world_ref.as_ref() };
        w.storage
            .get(&ComponentId::of::<T>())?
            .get(&self.ent)
            .and_then(|b| b.as_any().downcast_ref())
    }

    pub fn get_mut<T: Component + 'static>(&mut self) -> Option<&mut T> {
        let w = unsafe { self.world_ref.as_mut() };
        w.storage
            .get_mut(&ComponentId::of::<T>())?
            .get_mut(&self.ent)
            .and_then(|b| b.as_any_mut().downcast_mut())
    }
}
