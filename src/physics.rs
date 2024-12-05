use bitflags::bitflags;
use glam::Vec2;
use roast2d_derive::Component;

use crate::collision::handle_trace_result;
use crate::ecs::entity_ref::EntMut;
use crate::entities::Commands;
use crate::prelude::{CollisionMap, Ent};
use crate::trace::trace;
use crate::{ecs::world::World, engine::Engine};

use super::transform::Transform;

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

impl EntPhysics {
    pub fn is_at_least(self, physics: EntPhysics) -> bool {
        self.bits() >= physics.bits()
    }

    pub fn is_collide_mode(self, mode: EntCollidesMode) -> bool {
        self.bits() & mode.bits() != 0
    }
}

#[derive(Component)]
pub struct Physics {
    pub physics: EntPhysics,
    pub on_ground: bool,
    pub vel: Vec2,
    pub accel: Vec2,
    pub friction: Vec2,
    pub gravity: f32,
    pub mass: f32,
    pub restitution: f32,
    pub max_ground_normal: f32,
    pub min_slide_normal: f32,
    pub group: EntGroup,
    pub check_against: EntGroup,
}

impl Default for Physics {
    fn default() -> Self {
        Physics {
            on_ground: false,
            physics: EntPhysics::NONE,
            group: EntGroup::NONE,
            check_against: EntGroup::NONE,
            vel: Vec2::default(),
            accel: Vec2::default(),
            friction: Vec2::default(),
            gravity: 1.0,
            mass: 1.0,
            restitution: 0.0,
            max_ground_normal: 0.69, // cosf(to_radians(46))
            min_slide_normal: 1.0,   // cosf(to_radians(0))
        }
    }
}

/// Entity base update, handle physics velocities
pub(crate) fn entity_base_update(g: &mut Engine, w: &mut World, ent: Ent) {
    w.with_resource::<CollisionMap, _, _>(|w, map| {
        w.with_resource::<Commands, _, _>(|w, commands| {
            let Ok(mut ent) = w.get_mut(ent) else {
                return;
            };
            let Ok(phy) = ent.get_mut::<Physics>() else {
                return;
            };
            if !phy.physics.contains(EntPhysics::MOVE) {
                return;
            }
            // Integrate velocity
            let vel = phy.vel;
            phy.vel.y += g.gravity * phy.gravity * g.tick;
            let fric = Vec2::new(
                (phy.friction.x * g.tick).min(1.0),
                (phy.friction.y * g.tick).min(1.0),
            );
            phy.vel = phy.vel + (phy.accel * g.tick - phy.vel * fric);
            let vstep = (vel + phy.vel) * (g.tick * 0.5);
            phy.on_ground = false;
            entity_move(g, map, commands, &mut ent, vstep);
        });
    });
}

// Move entity
pub fn entity_move(
    _g: &mut Engine,
    map: &CollisionMap,
    commands: &mut Commands,
    ent: &mut EntMut,
    vstep: Vec2,
) {
    if ent
        .get::<Physics>()
        .is_ok_and(|phy| phy.physics.contains(EntPhysics::WORLD))
    {
        let Ok(transform) = ent.get::<Transform>() else {
            return;
        };
        let t = trace(
            map,
            transform.pos,
            vstep,
            transform.scaled_size(),
            transform.angle,
        );
        handle_trace_result(commands, ent, t.clone());
        // The previous trace was stopped short and we still have some velocity
        // left? Do a second trace with the new velocity. this allows us
        // to slide along tiles;
        if t.length < 1. {
            let rotated_normal = Vec2::new(-t.normal.y, t.normal.x);
            let vel_along_normal = vstep.dot(rotated_normal);

            if vel_along_normal != 0. {
                let remaining = 1. - t.length;
                let vstep2 = rotated_normal * (vel_along_normal * remaining);
                let Ok(transform) = ent.get::<Transform>() else {
                    return;
                };
                let t2 = trace(
                    map,
                    transform.pos,
                    vstep2,
                    transform.scaled_size(),
                    transform.angle,
                );
                handle_trace_result(commands, ent, t2);
            }
        }
    } else if let Ok(transform) = ent.get_mut::<Transform>() {
        transform.pos += vstep;
    }
}
