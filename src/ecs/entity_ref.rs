use std::any::type_name;

use crate::errors::Error;

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

    pub fn get<T: Component + 'static>(&'w self) -> Result<&'w T, Error> {
        let w = unsafe { self.world_ref.as_ref() };
        w.storage
            .get(&ComponentId::of::<T>())
            .ok_or(Error::NoComponent)?
            .get(&self.ent)
            .and_then(|b| b.as_any().downcast_ref())
            .ok_or(Error::NoComponent)
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

    pub fn add_by_name(&mut self, name: &str) -> Option<&mut Self> {
        let w = unsafe { self.world_ref.as_mut() };
        let component_id = w.get_component_id_by_name(name)?;
        let component = w.new_component(&component_id)?;
        w.storage
            .entry(component_id)
            .or_default()
            .insert(self.ent, component)?;
        Some(self)
    }

    pub fn get<T: Component + 'static>(&self) -> Result<&T, Error> {
        let w = unsafe { self.world_ref.as_ref() };
        w.storage
            .get(&ComponentId::of::<T>())
            .ok_or(Error::NoComponent)?
            .get(&self.ent)
            .and_then(|b| b.as_any().downcast_ref())
            .ok_or(Error::NoComponent)
    }

    pub fn get_mut<T: Component + 'static>(&mut self) -> Result<&mut T, Error> {
        let w = unsafe { self.world_ref.as_mut() };
        w.storage
            .get_mut(&ComponentId::of::<T>())
            .ok_or(Error::NoComponent)?
            .get_mut(&self.ent)
            .and_then(|b| b.as_any_mut().downcast_mut())
            .ok_or(Error::NoComponent)
    }
}
