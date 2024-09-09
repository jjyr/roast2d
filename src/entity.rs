use std::any::{Any, TypeId};

use bitflags::bitflags;
use dyn_clone::DynClone;
use glam::Vec2;

use crate::{
    animation::Animation, collision::calc_bounds, engine::Engine, trace::Trace, types::Rect,
    world::World,
};

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy)]
    pub struct EntPhysics: u8 {
        const NONE = 0;
        // Move the entity according to its velocity, but don't collide
        const MOVE = 1 << 0;

        // Move the entity and collide with the collision_map
        const WORLD = EntPhysics::MOVE.bits() | EntCollidesMode::WORLD.bits();

        // Move the entity, collide with the collision_map and other entities, but
        // only those other entities that have matching physics:
        // In ACTIVE vs. LITE or FIXED vs. ANY collisions, only the "weak" entity
        // moves, while the other one stays fixed. In ACTIVE vs. ACTIVE and ACTIVE
        // vs. PASSIVE collisions, both entities are moved. LITE or PASSIVE entities
        // don't collide with other LITE or PASSIVE entities at all. The behaiviour
        // for FIXED vs. FIXED collisions is undefined.
        const LITE = EntPhysics::WORLD.bits() | EntCollidesMode::LITE.bits();
        const PASSIVE = EntPhysics::WORLD.bits() | EntCollidesMode::PASSIVE.bits();
        const ACTIVE = EntPhysics::WORLD.bits() | EntCollidesMode::ACTIVE.bits();
        const FIXED = EntPhysics::WORLD.bits() | EntCollidesMode::FIXED.bits();
    }
}

impl EntPhysics {
    pub fn is_at_least(self, physics: EntPhysics) -> bool {
        self.bits() >= physics.bits()
    }

    pub fn is_collide_mode(self, mode: EntCollidesMode) -> bool {
        self.bits() & mode.bits() != 0
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy)]
    pub struct EntCollidesMode: u8 {
        const WORLD = 1 << 1;
        const LITE = 1 << 4;
        const PASSIVE = 1 << 5;
        const ACTIVE = 1 << 6;
        const FIXED = 1 << 7;
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy)]
    pub struct EntGroup: u8 {
        const NONE = 0;
        const PLAYER = 1 << 0;
        const NPC = 1 << 1;
        const ENEMY = 1 << 2;
        const ITEM = 1 << 3;
        const PROJECTILE = 1 << 4;
        const PICKUP = 1 << 5;
        const BREAKABLE = 1 << 6;
    }
}

/// Entity type
#[derive(Hash, PartialEq, Eq, Clone)]
pub struct EntTypeId(pub TypeId);

impl EntTypeId {
    pub fn of<T: 'static>() -> Self {
        Self(TypeId::of::<T>())
    }

    pub fn is<T: 'static>(&self) -> bool {
        Self::of::<T>().0 == self.0
    }
}

impl From<TypeId> for EntTypeId {
    fn from(value: TypeId) -> Self {
        Self(value)
    }
}

/// Entity
pub struct Ent {
    /// Unique EntRef
    pub ent_ref: EntRef,
    pub alive: bool,
    pub on_ground: bool,
    pub draw_order: u32,
    pub ent_type: EntTypeId,
    pub physics: EntPhysics,
    pub group: EntGroup,
    pub check_against: EntGroup,
    pub pos: Vec2,
    pub scale: Vec2,
    /// Angle in radians
    pub angle: f32,
    pub size: Vec2,
    pub vel: Vec2,
    pub accel: Vec2,
    pub friction: Vec2,
    pub offset: Vec2,
    pub name: String,
    pub health: f32,
    pub gravity: f32,
    pub mass: f32,
    pub restitution: f32,
    pub max_ground_normal: f32,
    pub min_slide_normal: f32,
    pub anim: Option<Animation>,
    pub(crate) instance: Option<Box<dyn EntType>>,
}

impl Ent {
    pub(crate) fn new(id: u16, ent_type: EntTypeId, instance: Box<dyn EntType>, pos: Vec2) -> Self {
        let instance = Some(instance);
        let ent_ref = EntRef { id, index: 0 };
        Ent {
            ent_ref,
            alive: true,
            on_ground: false,
            draw_order: 0,
            ent_type,
            physics: EntPhysics::NONE,
            group: EntGroup::NONE,
            check_against: EntGroup::NONE,
            pos,
            size: Vec2 { x: 8.8, y: 8.8 },
            vel: Vec2::default(),
            accel: Vec2::default(),
            friction: Vec2::default(),
            offset: Vec2::default(),
            name: Default::default(),
            health: 0.0,
            gravity: 1.0,
            mass: 1.0,
            restitution: 0.0,
            max_ground_normal: 0.69, // cosf(to_radians(46))
            min_slide_normal: 1.0,   // cosf(to_radians(0))
            anim: None,
            instance,
            scale: Vec2::splat(1.0),
            angle: 0.0,
        }
    }

    pub(crate) fn is_valid(&self, ent_ref: EntRef) -> bool {
        self.alive && self.ent_ref.id == ent_ref.id
    }

    pub fn scaled_size(&self) -> Vec2 {
        self.size * self.scale
    }

    pub fn bounds(&self) -> Rect {
        let half_size = self.scaled_size() * 0.5;
        calc_bounds(self.pos, half_size, self.angle)
    }
}

/// Entity ref
/// Use this to get entity from engine.
/// The index of EntityRef may be changed due to reordering,
///  so it is suggest to only use id to build relation.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct EntRef {
    pub(crate) id: u16,
    pub(crate) index: u16,
}

impl Default for EntRef {
    fn default() -> Self {
        Self {
            id: u16::MAX,
            index: u16::MAX,
        }
    }
}

/// EntityDef
pub trait EntType: DynClone {
    /// Load an entity type
    ///
    /// This function called only once on EntityType registration.
    /// Assets loading should be done in this callback,
    /// all Entity instances are cloned from this one when spawn is called.
    fn load(_eng: &mut Engine, _w: &mut World) -> Self
    where
        Self: Sized;

    /// Initialize an entity
    fn init(&mut self, _eng: &mut Engine, _w: &mut World, _ent: EntRef) {}

    /// Load entity settings
    fn settings(
        &mut self,
        _eng: &mut Engine,
        _w: &mut World,
        _ent: EntRef,
        _settings: serde_json::Value,
    ) {
    }

    /// Update callback is called before the entity_base_update
    fn update(&mut self, _eng: &mut Engine, _w: &mut World, _ent: EntRef) {}

    /// Post update callback is called after the entity_base_update
    fn post_update(&mut self, _eng: &mut Engine, _w: &mut World, _ent: EntRef) {}

    // Draw entity anim
    fn draw(&self, eng: &mut Engine, w: &mut World, ent: EntRef, viewport: Vec2) {
        let Some(ent) = w.get(ent) else {
            return;
        };
        if let Some(anim) = ent.anim.as_ref() {
            eng.render.borrow_mut().draw_image(
                &anim.sheet,
                (ent.pos - viewport) - ent.offset,
                Some(ent.scale),
                Some(ent.angle),
            );
        }
    }

    /// Called when entity is removed through kill
    fn kill(&mut self, _eng: &mut Engine, _w: &mut World, _ent: EntRef) {}

    /// Called if one entity is touched by another entity
    fn touch(&mut self, _eng: &mut Engine, _w: &mut World, _ent: EntRef, _other: EntRef) {}

    /// Called when two entity are collide
    fn collide(
        &mut self,
        _eng: &mut Engine,
        _w: &mut World,
        _ent: EntRef,
        _normal: Vec2,
        _trace: Option<&Trace>,
    ) {
    }

    /// Called when entity get damage
    fn damage(
        &mut self,
        eng: &mut Engine,
        w: &mut World,
        ent: EntRef,
        _other: EntRef,
        damage: f32,
    ) {
        let Some(ent) = w.get_mut(ent) else {
            return;
        };
        ent.health -= damage;
        if ent.health < 0.0 && ent.alive {
            let ent_ref = ent.ent_ref;
            eng.kill(ent_ref);
        }
    }

    /// Called when entity is triggerred by another entity
    fn trigger(&mut self, _eng: &mut Engine, _w: &mut World, _ent: EntRef, _other: EntRef) {}

    /// Called when entity recives a message
    fn message(&mut self, _eng: &mut Engine, _w: &mut World, _ent: EntRef, _data: Box<dyn Any>) {}
}
