use std::{any::type_name, cell::UnsafeCell, collections::HashMap, rc::Rc};

use crate::{
    entity::{Ent, EntRef, EntType, EntTypeId},
    sorts::insertion_sort_by_key,
    types::SweepAxis,
};

/// A mutable reference
pub(crate) struct Mut<T: ?Sized>(UnsafeCell<T>);

impl<T> Mut<T> {
    pub(crate) fn new(v: T) -> Mut<T> {
        Self(UnsafeCell::new(v))
    }
}

impl<T: ?Sized> Mut<T> {
    pub(crate) unsafe fn get(&self) -> &T {
        self.0.get().as_ref().unwrap()
    }

    #[allow(clippy::mut_from_ref)]
    pub(crate) unsafe fn get_mut(&self) -> &mut T {
        self.0.get().as_mut().unwrap()
    }
}

/// World contains entities
#[derive(Default)]
pub struct World {
    /// Unique id counter
    unique_id: u16,
    /// This field may be reorder
    entities: Vec<EntRef>,
    /// Allocated entities count
    alloced: usize,
    /// Fixed storage of entities, the index is unchanged
    entities_storage: Vec<Mut<Ent>>,
    /// Entity Types
    entity_types: HashMap<EntTypeId, Rc<dyn EntType>>,
    /// Name to Entity Types
    name_to_entity_types: HashMap<String, EntTypeId>,
}

impl World {
    /// Get an entity type instance
    pub(crate) fn get_ent_type_instance(&self, type_id: &EntTypeId) -> Option<Box<dyn EntType>> {
        self.entity_types.get(type_id).map(|ent_type| {
            let t = ent_type.as_ref();
            dyn_clone::clone_box(t)
        })
    }

    pub(crate) fn add_ent_type<T: EntType + Clone + 'static>(&mut self, ent_type: Rc<dyn EntType>) {
        let type_id = EntTypeId::of::<T>();
        self.entity_types.insert(type_id.clone(), ent_type);
        let name = type_name::<T>()
            .split("::")
            .last()
            .expect("can't get name of entity type");
        self.name_to_entity_types.insert(name.to_string(), type_id);
    }

    pub(crate) fn get_type_id_by_name(&self, name: &str) -> Option<&EntTypeId> {
        self.name_to_entity_types.get(name)
    }

    pub(crate) fn next_unique_id(&self) -> u16 {
        self.unique_id
    }

    pub(crate) fn alloced(&self) -> usize {
        self.alloced
    }

    /// Get an entity by ref
    pub(crate) fn get_nth(&self, n: usize) -> Option<&EntRef> {
        self.entities.get(n)
    }

    /// Get an entity by ref
    pub(crate) fn swap_remove(&mut self, n: usize) {
        self.entities.swap_remove(n);
        self.alloced -= 1;
    }

    pub(crate) fn sort_entities_for_sweep(&mut self, sweep_axis: SweepAxis) {
        let mut entities = core::mem::take(&mut self.entities);
        insertion_sort_by_key(&mut entities, |ent_ref| {
            let ent_bounds = self.get(*ent_ref).unwrap().bounds();
            sweep_axis.get(ent_bounds.min) as usize
        });
        let _ = core::mem::replace(&mut self.entities, entities);
    }

    /// Get an entity by ref
    pub(crate) fn get_unchecked(&self, ent_ref: EntRef) -> Option<&Ent> {
        self.entities_storage
            .get(ent_ref.index as usize)
            .map(|ent| unsafe { ent.get() })
    }

    /// Get an entity by ref
    pub fn get(&self, ent_ref: EntRef) -> Option<&Ent> {
        if let Some(ent) = self.entities_storage.get(ent_ref.index as usize) {
            let ent = unsafe { ent.get() };
            if ent.is_valid(ent_ref) {
                return Some(ent);
            }
        }
        None
    }

    /// Get an entity by ref
    pub fn get_mut(&mut self, ent_ref: EntRef) -> Option<&mut Ent> {
        if let Some(ent) = self.entities_storage.get(ent_ref.index as usize) {
            let ent = unsafe { ent.get_mut() };
            if ent.is_valid(ent_ref) {
                return Some(ent);
            }
        }
        None
    }

    /// Get an entity by ref
    pub fn get_many<const N: usize>(&self, ent_refs: [EntRef; N]) -> [Option<&Ent>; N] {
        ent_refs.map(|ent_ref| {
            if let Some(ent) = self.entities_storage.get(ent_ref.index as usize) {
                let ent = unsafe { ent.get() };
                if ent.is_valid(ent_ref) {
                    return Some(ent);
                }
            }
            None
        })
    }

    /// Get an entity by ref
    pub fn get_many_mut<const N: usize>(&mut self, ent_refs: [EntRef; N]) -> [Option<&mut Ent>; N] {
        ent_refs.map(|ent_ref| {
            if let Some(ent) = self.entities_storage.get(ent_ref.index as usize) {
                let ent = unsafe { ent.get_mut() };
                if ent.is_valid(ent_ref) {
                    return Some(ent);
                }
            }
            None
        })
    }

    /// Get an entity by ref
    pub fn many<const N: usize>(&self, ent_refs: [EntRef; N]) -> [&Ent; N] {
        ent_refs.map(|ent_ref| {
            let ent = self
                .entities_storage
                .get(ent_ref.index as usize)
                .expect("ent");
            let ent = unsafe { ent.get() };
            if !ent.is_valid(ent_ref) {
                panic!("invalid ent");
            }
            ent
        })
    }

    /// Get an entity by ref
    pub fn many_mut<const N: usize>(&mut self, ent_refs: [EntRef; N]) -> [&mut Ent; N] {
        ent_refs.map(|ent_ref| {
            let ent = self
                .entities_storage
                .get(ent_ref.index as usize)
                .expect("ent");
            let ent = unsafe { ent.get_mut() };
            if !ent.is_valid(ent_ref) {
                panic!("invalid ent");
            }
            ent
        })
    }

    /// Get an entity by ref
    pub fn entities(&self) -> impl Iterator<Item = &Ent> {
        self.entities
            .iter()
            .take(self.alloced)
            .filter_map(|ent_ref| self.get(*ent_ref))
    }

    /// Spawn a new entity
    pub(crate) fn spawn(&mut self, mut ent: Ent) -> EntRef {
        let id = self.unique_id;
        assert_eq!(ent.ent_ref.id, id, "expect unique_id");

        let index: u16;
        if self.entities.len() > self.alloced {
            // reuse slot
            index = self.entities[self.alloced].index;
            ent.ent_ref.index = index;
            self.entities_storage[index as usize] = Mut::new(ent);
        } else {
            // alloc slot
            index = self.entities_storage.len() as u16;
            ent.ent_ref.index = index;
            self.entities.push(ent.ent_ref);
            self.entities_storage.push(Mut::new(ent));
            self.alloced += 1;
        };

        self.unique_id += 1;

        EntRef { id, index }
    }

    pub(crate) fn reset_entities(&mut self) {
        self.entities.clear();
        self.entities_storage.clear();
        self.alloced = 0;
    }
}
