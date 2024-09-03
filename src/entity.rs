use std::{
    any::{Any, TypeId},
    collections::HashMap,
    rc::Rc,
};

use bitflags::bitflags;
use dyn_clone::DynClone;
use glam::Vec2;

use crate::{
    animation::Animation,
    engine::Engine,
    trace::{trace, Trace},
    types::{Mut, Rect},
};

/// Call with ent instance
macro_rules! with_ent {
    ($ent: expr, $f: expr) => {
        if let Some(mut instance) = $ent.instance.take() {
            $f(&mut instance);
            $ent.instance.replace(instance);
        } else {
            log::error!("Can't get entity instance {:?}", $ent.ent_ref)
        }
    };
}

pub(crate) use with_ent;

const ENTITY_MIN_BOUNCE_VELOCITY: f32 = 10.0;

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy)]
    pub struct EntityPhysics: u8 {
        const NONE = 0;
        // Move the entity according to its velocity, but don't collide
        const MOVE = 1 << 0;

        // Move the entity and collide with the collision_map
        const WORLD = EntityPhysics::MOVE.bits() | EntityCollidesMode::WORLD.bits();

        // Move the entity, collide with the collision_map and other entities, but
        // only those other entities that have matching physics:
        // In ACTIVE vs. LITE or FIXED vs. ANY collisions, only the "weak" entity
        // moves, while the other one stays fixed. In ACTIVE vs. ACTIVE and ACTIVE
        // vs. PASSIVE collisions, both entities are moved. LITE or PASSIVE entities
        // don't collide with other LITE or PASSIVE entities at all. The behaiviour
        // for FIXED vs. FIXED collisions is undefined.
        const LITE = EntityPhysics::WORLD.bits() | EntityCollidesMode::LITE.bits();
        const PASSIVE = EntityPhysics::WORLD.bits() | EntityCollidesMode::PASSIVE.bits();
        const ACTIVE = EntityPhysics::WORLD.bits() | EntityCollidesMode::ACTIVE.bits();
        const FIXED = EntityPhysics::WORLD.bits() | EntityCollidesMode::FIXED.bits();
    }
}

impl EntityPhysics {
    pub fn is_at_least(self, physics: EntityPhysics) -> bool {
        self.bits() >= physics.bits()
    }

    pub fn is_collide_mode(self, mode: EntityCollidesMode) -> bool {
        self.bits() & mode.bits() != 0
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy)]
    pub struct EntityCollidesMode: u8 {
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
    pub struct EntityGroup: u8 {
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
pub struct EntityTypeId(pub TypeId);

impl EntityTypeId {
    pub fn of<T: 'static>() -> Self {
        Self(TypeId::of::<T>())
    }

    pub fn is<T: 'static>(&self) -> bool {
        Self::of::<T>().0 == self.0
    }
}

impl From<TypeId> for EntityTypeId {
    fn from(value: TypeId) -> Self {
        Self(value)
    }
}

/// Entity
pub struct Entity {
    pub ent_ref: EntityRef,
    pub alive: bool,
    pub on_ground: bool,
    pub draw_order: u32,
    pub ent_type: EntityTypeId,
    pub physics: EntityPhysics,
    pub group: EntityGroup,
    pub check_against: EntityGroup,
    pub pos: Vec2,
    pub scale: Vec2,
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
    pub(crate) instance: Option<Box<dyn EntityType>>,
}

impl Entity {
    pub(crate) fn new(
        id: u16,
        ent_type: EntityTypeId,
        instance: Box<dyn EntityType>,
        pos: Vec2,
    ) -> Self {
        let instance = Some(instance);
        let ent_ref = EntityRef { id, index: 0 };
        Entity {
            ent_ref,
            alive: true,
            on_ground: false,
            draw_order: 0,
            ent_type,
            physics: EntityPhysics::NONE,
            group: EntityGroup::NONE,
            check_against: EntityGroup::NONE,
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

    pub(crate) fn is_touching(&self, other: &Entity) -> bool {
        let self_bound = self.bounds();
        let other_bound = other.bounds();
        !(self_bound.min.x > other_bound.max.x
            || self_bound.max.x < other_bound.min.x
            || self_bound.min.y > other_bound.max.y
            || self_bound.max.y < other_bound.min.y)
    }

    pub fn scaled_size(&self) -> Vec2 {
        self.size * self.scale
    }

    pub fn bounds(&self) -> Rect {
        let half_size = self.scaled_size() * 0.5;
        let min = self.pos - half_size;
        let max = self.pos + half_size;
        Rect { min, max }
    }
}

/// Entity ref
/// Use this to get entity from engine.
/// The index of EntityRef may be changed due to reordering,
///  so it is suggest to only use id to build relation.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct EntityRef {
    id: u16,
    index: u16,
}

/// EntityDef
pub trait EntityType: DynClone {
    /// Load an entity type
    ///
    /// This function called only once on EntityType registration.
    /// Assets loading should be done in this callback,
    /// all Entity instances are cloned from this one when spawn is called.
    fn load(_eng: &mut Engine) -> Self
    where
        Self: Sized;

    /// Initialize an entity
    fn init(&mut self, _eng: &mut Engine, _ent: &mut Entity) {}

    /// Load entity settings
    fn settings(&mut self, _eng: &mut Engine, _ent: &mut Entity, _settings: serde_json::Value) {}

    /// Update callback is called before the entity_base_update
    fn update(&mut self, _eng: &mut Engine, _ent: &mut Entity) {}

    /// Post update callback is called after the entity_base_update
    fn post_update(&mut self, _eng: &mut Engine, _ent: &mut Entity) {}

    // Draw entity anim
    fn draw(&self, eng: &mut Engine, ent: &mut Entity, viewport: Vec2) {
        if let Some(anim) = ent.anim.as_mut() {
            anim.draw(
                &mut eng.render,
                (ent.pos - viewport) - ent.offset,
                Some(ent.scale),
                Some(ent.angle),
            );
        }
    }

    /// Called when entity is removed through kill
    fn kill(&mut self, _eng: &mut Engine, _ent: &mut Entity) {}

    /// Called if one entity is touched by another entity
    fn touch(&mut self, _eng: &mut Engine, _ent: &mut Entity, _other: &mut Entity) {}

    /// Called when two entity are collide
    fn collide(
        &mut self,
        _eng: &mut Engine,
        _ent: &mut Entity,
        _normal: Vec2,
        _trace: Option<&Trace>,
    ) {
    }

    /// Called when entity get damage
    fn damage(&mut self, eng: &mut Engine, ent: &mut Entity, _other: &mut Entity, damage: f32) {
        ent.health -= damage;
        if ent.health < 0.0 && ent.alive {
            eng.kill(ent.ent_ref);
        }
    }

    /// Called when entity is triggerred by another entity
    fn trigger(&mut self, _eng: &mut Engine, _ent: &mut Entity, _other: &mut Entity) {}

    /// Called when entity recives a message
    fn message(
        &mut self,
        _eng: &mut Engine,
        _ent: &mut Entity,
        _message: u32,
        _data: Box<dyn Any>,
    ) {
    }
}

/// World contains entities
#[derive(Default)]
pub struct World {
    /// Unique id counter
    pub(crate) unique_id: u16,
    /// This field may be reorder
    pub(crate) entities: Vec<Mut<Entity>>,
    /// Allocated entities count
    pub(crate) alloced: usize,
    /// Fixed storage of entities, the index is unchanged
    pub(crate) entities_storage: Vec<Mut<Entity>>,
    /// Entity Types
    pub(crate) entity_types: HashMap<EntityTypeId, Rc<dyn EntityType>>,
    /// Name to Entity Types
    pub(crate) name_to_entity_types: HashMap<String, EntityTypeId>,
}

impl World {
    /// Get an entity type instance
    pub(crate) fn get_entity_type_instance(
        &self,
        type_id: &EntityTypeId,
    ) -> Option<Box<dyn EntityType>> {
        self.entity_types.get(type_id).map(|ent_type| {
            let t = ent_type.as_ref();
            dyn_clone::clone_box(t)
        })
    }

    /// Get an entity by ref
    pub fn get(&self, ent_ref: EntityRef) -> Option<Mut<Entity>> {
        self.entities_storage
            .get(ent_ref.index as usize)
            .filter(|ent| {
                let ent = ent.borrow();
                ent.alive && ent.ent_ref.id == ent_ref.id
            })
            .cloned()
    }

    /// Get an entity by ref
    pub fn entities(&self) -> impl Iterator<Item = &Mut<Entity>> {
        self.entities.iter()
    }

    /// Spawn a new entity
    pub(crate) fn spawn(&mut self, mut ent: Entity) -> EntityRef {
        let id = self.unique_id;
        assert_eq!(ent.ent_ref.id, id, "expect unique_id");

        let index: u16;
        if self.entities.len() > self.alloced {
            // reuse slot
            let old_ent = self.entities[self.alloced].clone();
            index = old_ent.borrow().ent_ref.index;
            ent.ent_ref.index = index;
            self.entities_storage[index as usize] = Mut::new(ent);
        } else {
            // alloc slot
            index = self.entities_storage.len() as u16;
            ent.ent_ref.index = index;
            let ent = Mut::new(ent);
            self.entities_storage.push(ent.clone());
            self.entities.push(ent);
            self.alloced += 1;
        };

        self.unique_id += 1;

        EntityRef { id, index }
    }

    pub(crate) fn reset_entities(&mut self) {
        self.entities.clear();
        self.entities_storage.clear();
        self.alloced = 0;
    }
}

/// Resolve entity collision
pub(crate) fn resolve_collision(eng: &mut Engine, a: &mut Entity, b: &mut Entity) {
    let a_bound = a.bounds();
    let b_bound = b.bounds();
    let overlap_x: f32 = if a_bound.min.x < b_bound.min.x {
        a_bound.max.x - b_bound.min.x
    } else {
        b_bound.max.x - a_bound.min.x
    };
    let overlap_y: f32 = if a_bound.min.y < b_bound.min.y {
        a_bound.max.y - b_bound.min.y
    } else {
        b_bound.max.y - a_bound.min.y
    };

    let a_move;
    let b_move;
    if a.physics.is_collide_mode(EntityCollidesMode::LITE)
        || b.physics.is_collide_mode(EntityCollidesMode::FIXED)
    {
        a_move = 1.0;
        b_move = 0.0;
    } else if a.physics.is_collide_mode(EntityCollidesMode::FIXED)
        || b.physics.is_collide_mode(EntityCollidesMode::LITE)
    {
        a_move = 0.0;
        b_move = 1.0;
    } else {
        let total_mass = a.mass + b.mass;
        a_move = b.mass / total_mass;
        b_move = a.mass / total_mass;
    }

    if overlap_y > overlap_x {
        if a_bound.min.x < b_bound.min.x {
            entities_separate_on_x_axis(eng, a, b, a_move, b_move, overlap_x);
            eng.collide(a, Vec2::new(-1.0, 0.0), None);
            eng.collide(b, Vec2::new(1.0, 0.0), None);
        } else {
            entities_separate_on_x_axis(eng, b, a, b_move, a_move, overlap_x);
            eng.collide(a, Vec2::new(1.0, 0.0), None);
            eng.collide(b, Vec2::new(-1.0, 0.0), None);
        }
    } else if a_bound.min.y < b_bound.min.y {
        entities_separate_on_y_axis(eng, a, b, a_move, b_move, overlap_y, eng.tick);
        eng.collide(a, Vec2::new(0.0, -1.0), None);
        eng.collide(b, Vec2::new(0.0, 1.0), None);
    } else {
        entities_separate_on_y_axis(eng, b, a, b_move, a_move, overlap_y, eng.tick);
        eng.collide(a, Vec2::new(0.0, 1.0), None);
        eng.collide(b, Vec2::new(0.0, -1.0), None);
    }
}

pub(crate) fn entities_separate_on_x_axis(
    eng: &mut Engine,
    left: &mut Entity,
    right: &mut Entity,
    left_move: f32,
    right_move: f32,
    overlap: f32,
) {
    let impact_velocity = left.vel.x - right.vel.x;
    if left_move > 0.0 {
        left.vel.x = right.vel.x * left_move + left.vel.x * right_move;
        let bounce = impact_velocity * left.restitution;
        if bounce > ENTITY_MIN_BOUNCE_VELOCITY {
            left.vel.x -= bounce;
        }
        entity_move(eng, left, Vec2::new(-overlap * left_move, 0.0));
    }

    if right_move > 0.0 {
        right.vel.x = left.vel.x * right_move + right.vel.x * left_move;
        let bounce = impact_velocity * right.restitution;
        if bounce > ENTITY_MIN_BOUNCE_VELOCITY {
            right.vel.x += bounce;
        }
        entity_move(eng, right, Vec2::new(overlap * right_move, 0.0));
    }
}

pub(crate) fn entities_separate_on_y_axis(
    eng: &mut Engine,
    top: &mut Entity,
    bottom: &mut Entity,
    mut top_move: f32,
    mut bottom_move: f32,
    overlap: f32,
    ticks: f32,
) {
    if bottom.on_ground && top_move > 0.0 {
        top_move = 1.0;
        bottom_move = 0.0;
    }

    let impact_velocity = top.vel.y - bottom.vel.y;
    let top_vel_y = top.vel.y;

    if top_move > 0.0 {
        top.vel.y = top.vel.y * bottom_move + bottom.vel.y * top_move;
        let mut move_x = 0.0;
        let bounce = impact_velocity * top.restitution;
        if bounce > ENTITY_MIN_BOUNCE_VELOCITY {
            top.vel.y -= bounce;
        } else {
            top.on_ground = true;
            move_x = bottom.vel.x * ticks;
        }
        entity_move(eng, top, Vec2::new(move_x, -overlap * top_move));
    }

    if bottom_move > 0.0 {
        bottom.vel.y = bottom.vel.y * top_move + top_vel_y * bottom_move;
        let bounce = impact_velocity * bottom.restitution;
        if bounce > ENTITY_MIN_BOUNCE_VELOCITY {
            bottom.vel.y += bounce;
        }
        entity_move(eng, bottom, Vec2::new(0.0, overlap * bottom_move));
    }
}

pub(crate) fn entity_move(eng: &mut Engine, ent: &mut Entity, vstep: Vec2) {
    if ent.physics.contains(EntityPhysics::WORLD) && eng.collision_map.is_some() {
        let map = eng.collision_map.as_ref().unwrap();
        let t = trace(map, ent.pos, vstep, ent.scaled_size());
        handle_trace_result(eng, ent, &t);
        // The previous trace was stopped short and we still have some velocity
        // left? Do a second trace with the new velocity. this allows us
        // to slide along tiles;
        if t.length < 1. {
            let rotated_normal = Vec2::new(-t.normal.y, t.normal.x);
            let vel_along_normal = vstep.dot(rotated_normal);

            if vel_along_normal != 0. {
                let remaining = 1. - t.length;
                let vstep2 = rotated_normal * (vel_along_normal * remaining);
                let map = eng.collision_map.as_ref().unwrap();
                let t2 = trace(map, ent.pos, vstep2, ent.scaled_size());
                handle_trace_result(eng, ent, &t2);
            }
        }
    } else {
        ent.pos += vstep;
    }
}

fn handle_trace_result(eng: &mut Engine, ent: &mut Entity, t: &Trace) {
    ent.pos = t.pos;

    if t.tile == 0 {
        return;
    }

    with_ent!(ent, |instance: &mut Box<dyn EntityType>| {
        instance.collide(eng, ent, t.normal, Some(t));
    });

    // If this entity is bouncy, calculate the velocity against the
    // slope's normal (the dot product) and see if we want to bounce
    // back.
    if ent.restitution > 0. {
        let vel_against_normal = ent.vel.dot(t.normal);

        if vel_against_normal.abs() * ent.restitution > ENTITY_MIN_BOUNCE_VELOCITY {
            let vn = t.normal * vel_against_normal * 2.;
            ent.vel = (ent.vel - vn) * ent.restitution;
            return;
        }
    }

    // If this game has gravity, we may have to set the on_ground flag.
    if (eng.gravity != 0.0) && t.normal.y < -ent.max_ground_normal {
        ent.on_ground = true;

        // If we don't want to slide on slopes, we cheat a bit by
        // fudging the y velocity.
        if t.normal.y < -ent.min_slide_normal {
            ent.vel.y = ent.vel.x * t.normal.x;
        }
    }

    // Rotate the normal vector by 90° ([nx, ny] -> [-ny, nx]) to get
    // the slope vector and calculate the dot product with the velocity.
    // This is the velocity with which we will slide along the slope.
    let rotated_normal = Vec2::new(-t.normal.y, t.normal.x);
    let vel_along_normal = ent.vel.dot(rotated_normal);
    ent.vel = rotated_normal * vel_along_normal;
}
