use std::any::{Any, TypeId};

use bitflags::bitflags;
use dyn_clone::DynClone;
use glam::Vec2;

use crate::{
    animation::Animation,
    engine::Engine,
    trace::{trace, Trace},
    types::Rect,
    world::World,
};

const ENTITY_MIN_BOUNCE_VELOCITY: f32 = 10.0;

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

/// Resolve entity collision
pub(crate) fn resolve_collision(eng: &mut Engine, w: &mut World, a: EntRef, b: EntRef) {
    let [a, b] = w.many_mut([a, b]);
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
    if a.physics.is_collide_mode(EntCollidesMode::LITE)
        || b.physics.is_collide_mode(EntCollidesMode::FIXED)
    {
        a_move = 1.0;
        b_move = 0.0;
    } else if a.physics.is_collide_mode(EntCollidesMode::FIXED)
        || b.physics.is_collide_mode(EntCollidesMode::LITE)
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
            eng.collide(a.ent_ref, Vec2::new(-1.0, 0.0), None);
            eng.collide(b.ent_ref, Vec2::new(1.0, 0.0), None);
        } else {
            entities_separate_on_x_axis(eng, b, a, b_move, a_move, overlap_x);
            eng.collide(a.ent_ref, Vec2::new(1.0, 0.0), None);
            eng.collide(b.ent_ref, Vec2::new(-1.0, 0.0), None);
        }
    } else if a_bound.min.y < b_bound.min.y {
        entities_separate_on_y_axis(eng, a, b, a_move, b_move, overlap_y, eng.tick);
        eng.collide(a.ent_ref, Vec2::new(0.0, -1.0), None);
        eng.collide(b.ent_ref, Vec2::new(0.0, 1.0), None);
    } else {
        entities_separate_on_y_axis(eng, b, a, b_move, a_move, overlap_y, eng.tick);
        eng.collide(a.ent_ref, Vec2::new(0.0, 1.0), None);
        eng.collide(b.ent_ref, Vec2::new(0.0, -1.0), None);
    }
}

pub(crate) fn entities_separate_on_x_axis(
    eng: &mut Engine,
    left: &mut Ent,
    right: &mut Ent,
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
    top: &mut Ent,
    bottom: &mut Ent,
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

pub(crate) fn entity_move(eng: &mut Engine, ent: &mut Ent, vstep: Vec2) {
    if ent.physics.contains(EntPhysics::WORLD) && eng.collision_map.is_some() {
        let map = eng.collision_map.as_ref().unwrap();
        let t = trace(map, ent.pos, vstep, ent.scaled_size());
        handle_trace_result(eng, ent, t.clone());
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
                handle_trace_result(eng, ent, t2);
            }
        }
    } else {
        ent.pos += vstep;
    }
}

fn handle_trace_result(eng: &mut Engine, ent: &mut Ent, t: Trace) {
    ent.pos = t.pos;

    // FIXME call check collision rule
    if t.tile == 0 {
        return;
    }

    eng.collide(ent.ent_ref, t.normal, Some(t.clone()));

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
