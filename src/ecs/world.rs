use std::{
    any::{type_name, Any},
    cell::UnsafeCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::{ecs::entity::Ent, sorts::insertion_sort_by_key, types::SweepAxis};

use super::{
    component::{Component, ComponentId},
    entity_ref::{EntMut, EntRef},
    resource::Resource,
    unsafe_world_ref::UnsafeWorldRef,
};

/// World contains entities
#[derive(Default)]
pub struct World {
    /// Unique id counter
    unique_id: u32,
    /// Live Ents
    entities: HashSet<Ent>,
    /// Component storage
    pub(crate) storage: HashMap<ComponentId, HashMap<Ent, Box<dyn Component>>>,
    /// Name to Entity Types
    components_names: HashMap<String, ComponentId>,
    resources: HashMap<ComponentId, Box<dyn Resource>>,
}

impl World {
    pub(crate) fn init_component<T: Component + Clone + 'static>(&mut self) {
        let type_id = ComponentId::of::<T>();
        let name = type_name::<T>()
            .split("::")
            .last()
            .expect("can't get name of entity type");
        self.components_names.insert(name.to_string(), type_id);
    }

    pub(crate) fn get_component_by_name(&self, name: &str) -> Option<&ComponentId> {
        self.components_names.get(name)
    }

    fn to_unsafe_world_ref(&self) -> UnsafeWorldRef {
        UnsafeWorldRef::new_readonly(self)
    }

    fn to_unsafe_world_mut(&mut self) -> UnsafeWorldRef {
        UnsafeWorldRef::new_mutable(self)
    }

    /// Add Resource
    pub fn add_resource<T: Resource + 'static>(&mut self, resource: T) {
        let id = ComponentId::of::<T>();
        self.resources.insert(id, Box::new(resource));
    }

    /// Get Resource
    pub fn get_resource<T: Resource + 'static>(&self) -> Option<&T> {
        let id = ComponentId::of::<T>();
        self.resources
            .get(&id)
            .map(|r| unsafe { &*<*const _>::cast(r) })
    }

    /// Get Resource mut
    pub fn get_resource_mut<T: Resource + 'static>(&mut self) -> Option<&mut T> {
        let id = ComponentId::of::<T>();
        self.resources
            .get_mut(&id)
            .map(|r| unsafe { &mut *<*mut _>::cast(r) })
    }

    /// Get an entity ref
    pub fn get(&self, ent: Ent) -> Option<EntRef> {
        if self.entities.contains(&ent) {
            Some(EntRef::new(ent, self.to_unsafe_world_ref()))
        } else {
            None
        }
    }

    /// Get an entity by ref
    pub fn get_mut(&mut self, ent: Ent) -> Option<EntMut> {
        if self.entities.contains(&ent) {
            Some(EntMut::new(ent, self.to_unsafe_world_mut()))
        } else {
            None
        }
    }

    /// Get many entity
    pub fn get_many<const N: usize>(&self, ents: [Ent; N]) -> [Option<EntRef>; N] {
        ents.map(|ent| self.get(ent))
    }

    /// Get many entity mut
    pub fn get_many_mut<const N: usize>(&mut self, ents: [Ent; N]) -> [Option<EntMut>; N] {
        ents.map(|ent| {
            if self.entities.contains(&ent) {
                let world_ref = UnsafeWorldRef::new_readonly(self);
                Some(EntMut::new(ent, world_ref))
            } else {
                None
            }
        })
    }

    /// Get many entity
    pub fn many<const N: usize>(&self, ents: [Ent; N]) -> [EntRef; N] {
        ents.map(|ent| self.get(ent).expect("ent not exist"))
    }

    /// Get many entity mut
    pub fn many_mut<const N: usize>(&mut self, ents: [Ent; N]) -> [EntMut; N] {
        ents.map(|ent| {
            if self.entities.contains(&ent) {
                let world_ref = UnsafeWorldRef::new_readonly(self);
                EntMut::new(ent, world_ref)
            } else {
                panic!("ent not exist")
            }
        })
    }

    pub fn ents_count(&self) -> usize {
        self.entities.len()
    }

    /// Iterate entities
    pub fn iter_ents(&self) -> impl Iterator<Item = &Ent> {
        self.entities.iter()
    }

    /// Iterate entities
    pub fn iter_ent_refs(&self) -> impl Iterator<Item = EntRef> {
        self.entities.iter().map(|ent| {
            let world_ref = UnsafeWorldRef::new_readonly(self);
            EntRef::new(*ent, world_ref)
        })
    }

    /// Spawn a new entity
    pub fn spawn(&mut self) -> EntMut {
        let index = self.unique_id;
        self.unique_id = self.unique_id.checked_add(1).expect("Too many entities");
        let ent = Ent { index };
        self.entities.insert(ent);
        self.get_mut(ent).unwrap()
    }

    pub fn despawn(&mut self, ent: Ent) {
        self.entities.remove(&ent);
        for component_store in self.storage.values_mut() {
            component_store.remove(&ent);
        }
    }

    pub(crate) fn reset_entities(&mut self) {
        self.entities.clear();
        self.storage.clear();
    }
}
