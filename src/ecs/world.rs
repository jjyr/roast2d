use hashbrown::{HashMap, HashSet};
use std::any::type_name;

use crate::{ecs::entity::Ent, errors::Error};

use super::{
    component::{Component, ComponentId},
    entity_ref::{EntMut, EntRef},
    resource::Resource,
    unsafe_world_ref::UnsafeWorldRef,
};

/// World contains entities
#[derive(Default)]
pub struct World {
    /// Unique id
    unique_id: u32,
    /// Entities
    entities: HashSet<Ent>,
    /// Component storage
    pub(crate) storage: HashMap<ComponentId, HashMap<Ent, Box<dyn Component>>>,
    /// Resources
    resources: HashMap<ComponentId, Box<dyn Resource>>,
    /// Component by name
    component_by_name: HashMap<String, ComponentId>,
}

impl World {
    pub fn init_component<T: Component + 'static>(&mut self) {
        let component_id = ComponentId::of::<T>();
        let name = type_name::<T>()
            .split("::")
            .last()
            .expect("can't get name of component");
        // insert component name
        self.component_by_name
            .insert(name.to_string(), component_id.clone());
        // init storage
        let _ = self.storage.try_insert(component_id, Default::default());
    }

    pub fn get_component_id_by_name(&self, name: &str) -> Option<ComponentId> {
        self.component_by_name.get(name).cloned()
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

    /// Remove Resource
    pub fn remove_resource<T: Resource + 'static>(&mut self) -> Option<T> {
        let id = ComponentId::of::<T>();
        let r = self.resources.remove(&id)?;
        if let Ok(r) = r.into_any().downcast::<T>() {
            Some(*r)
        } else {
            None
        }
    }

    /// Temporarily remove resource
    pub fn with_resource<T: Resource + 'static, R, F: FnOnce(&mut World, &mut T) -> R>(
        &mut self,
        handle: F,
    ) -> R {
        let mut res = self.remove_resource::<T>().expect("with resource");
        let r = handle(self, &mut res);
        self.add_resource(res);
        r
    }

    /// Get Resource
    pub fn get_resource<T: Resource + 'static>(&self) -> Result<&T, Error> {
        let id = ComponentId::of::<T>();
        self.resources
            .get(&id)
            .and_then(|r| r.as_any().downcast_ref())
            .ok_or(Error::NoResource)
    }

    /// Get Resource mut
    pub fn get_resource_mut<T: Resource + 'static>(&mut self) -> Result<&mut T, Error> {
        let id = ComponentId::of::<T>();
        self.resources
            .get_mut(&id)
            .and_then(|r| r.as_any_mut().downcast_mut())
            .ok_or(Error::NoResource)
    }

    /// Get an entity ref
    pub fn get(&self, ent: Ent) -> Result<EntRef, Error> {
        if self.entities.contains(&ent) {
            Ok(EntRef::new(ent, self.to_unsafe_world_ref()))
        } else {
            Err(Error::NoEntity)
        }
    }

    /// Get an entity by ref
    pub fn get_mut(&mut self, ent: Ent) -> Result<EntMut, Error> {
        if self.entities.contains(&ent) {
            Ok(EntMut::new(ent, self.to_unsafe_world_mut()))
        } else {
            Err(Error::NoEntity)
        }
    }

    /// Get an entity ref
    pub fn ent(&self, ent: Ent) -> EntRef {
        self.get(ent).expect("entity")
    }

    /// Get an entity by ref
    pub fn ent_mut(&mut self, ent: Ent) -> EntMut {
        self.get_mut(ent).expect("entity_mut")
    }

    /// Get many entity
    pub fn get_many<const N: usize>(&self, ents: [Ent; N]) -> Result<Vec<EntRef>, Error> {
        ents.map(|ent| self.get(ent)).into_iter().collect()
    }

    /// Get many entity mut
    pub fn get_many_mut<const N: usize>(&mut self, ents: [Ent; N]) -> Result<Vec<EntMut>, Error> {
        ents.map(|ent| {
            if self.entities.contains(&ent) {
                let world_ref = UnsafeWorldRef::new_readonly(self);
                Ok(EntMut::new(ent, world_ref))
            } else {
                Err(Error::NoEntity)
            }
        })
        .into_iter()
        .collect()
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
    pub fn iter_ents_ref(&self) -> impl Iterator<Item = EntRef> {
        self.entities.iter().map(|ent| {
            let world_ref = UnsafeWorldRef::new_readonly(self);
            EntRef::new(*ent, world_ref)
        })
    }

    /// Iterate entities
    pub fn iter_ents_mut(&mut self) -> impl Iterator<Item = EntMut> {
        self.entities.iter().map(|ent| {
            let world_ref = UnsafeWorldRef::new_readonly(self);
            EntMut::new(*ent, world_ref)
        })
    }

    /// Iterate component
    pub fn iter_by<T: Component + 'static>(&self) -> impl Iterator<Item = &Ent> {
        let component_id = ComponentId::of::<T>();
        self.storage
            .get(&component_id)
            .map(|v| v.keys())
            .expect("No component, make sure init_component first")
    }

    /// Iterate component
    pub fn iter_ref_by<T: Component + 'static>(&self) -> impl Iterator<Item = EntRef> {
        let component_id = ComponentId::of::<T>();
        self.storage
            .get(&component_id)
            .map(|v| {
                v.keys().map(|ent| {
                    let world_ref = UnsafeWorldRef::new_readonly(self);
                    EntRef::new(*ent, world_ref)
                })
            })
            .expect("No component, make sure init_component first")
    }

    /// Iterate component
    pub fn iter_mut_by<T: Component + 'static>(&mut self) -> impl Iterator<Item = EntMut> {
        let component_id = ComponentId::of::<T>();
        self.storage
            .get(&component_id)
            .map(|v| {
                v.keys().map(|ent| {
                    let world_ref = UnsafeWorldRef::new_readonly(self);
                    EntMut::new(*ent, world_ref)
                })
            })
            .expect("No component, make sure init_component first")
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

    /// Remove entities
    pub fn clear_entities(&mut self) {
        self.entities.clear();
        self.storage.clear();
    }

    /// Remove entities and resources
    pub fn clear(&mut self) {
        self.clear_entities();
        self.resources.clear();
    }
}
