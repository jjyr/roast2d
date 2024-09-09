use std::f32::consts::PI;

use glam::Vec2;

const ENTITY_MIN_BOUNCE_VELOCITY: f32 = 10.0;

use crate::{
    engine::Engine,
    entity::{Ent, EntCollidesMode, EntPhysics, EntRef},
    trace::{trace, Trace},
    types::Rect,
    world::World,
};

// Move entity
pub fn entity_move(eng: &mut Engine, ent: &mut Ent, vstep: Vec2) {
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

pub(crate) fn calc_bounds(pos: Vec2, half_size: Vec2, angle: f32) -> Rect {
    const HF_PI: f32 = PI * 0.5;

    if angle == 0. || angle.abs() == PI {
        let min = pos - half_size;
        let max = pos + half_size;
        Rect { min, max }
    } else if angle.abs() == HF_PI {
        let half_size = Vec2 {
            x: half_size.y,
            y: half_size.x,
        };
        let min = pos - half_size;
        let max = pos + half_size;
        Rect { min, max }
    } else {
        let rot = Vec2::from_angle(angle);
        let p1 = pos + Vec2::new(half_size.x, -half_size.y);
        let p2 = pos + half_size;
        let p3 = pos + Vec2::new(-half_size.x, half_size.y);
        let p4 = pos - half_size;
        if angle > 0. && angle < HF_PI {
            let max_x = rot.rotate(p1).x;
            let min_x = rot.rotate(p3).x;
            let max_y = rot.rotate(p2).y;
            let min_y = rot.rotate(p4).y;
            Rect {
                min: Vec2::new(min_x, min_y),
                max: Vec2::new(max_x, max_y),
            }
        } else if angle > HF_PI && angle < PI {
            let max_x = rot.rotate(p4).x;
            let min_x = rot.rotate(p2).x;
            let max_y = rot.rotate(p1).y;
            let min_y = rot.rotate(p3).y;
            Rect {
                min: Vec2::new(min_x, min_y),
                max: Vec2::new(max_x, max_y),
            }
        } else if angle > -PI && angle < -HF_PI {
            let max_x = rot.rotate(p3).x;
            let min_x = rot.rotate(p1).x;
            let max_y = rot.rotate(p4).y;
            let min_y = rot.rotate(p2).y;
            Rect {
                min: Vec2::new(min_x, min_y),
                max: Vec2::new(max_x, max_y),
            }
        } else if angle > -HF_PI && angle < 0.0 {
            let max_x = rot.rotate(p2).x;
            let min_x = rot.rotate(p4).x;
            let max_y = rot.rotate(p3).y;
            let min_y = rot.rotate(p1).y;
            Rect {
                min: Vec2::new(min_x, min_y),
                max: Vec2::new(max_x, max_y),
            }
        } else {
            panic!("Unnormalized angle {angle}")
        }
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::calc_bounds;
    use super::Rect;

    #[test]
    fn test_calc_bounds() {
        let half_size = Vec2::new(2.0, 1.0);
        let b1 = calc_bounds(Vec2::ZERO, half_size, 0.0);
        let b2 = calc_bounds(Vec2::ZERO, half_size, 90f32.to_radians());
        let b3 = calc_bounds(Vec2::ZERO, half_size, 180f32.to_radians());
        let b4 = calc_bounds(Vec2::ZERO, half_size, -90f32.to_radians());
        let b5 = calc_bounds(Vec2::ZERO, half_size, -180f32.to_radians());

        assert_eq!(b1, b3);
        assert_eq!(b1, b5);
        assert_eq!(b2, b4);

        assert_eq!(
            b1,
            Rect {
                min: Vec2::new(-2., -1.),
                max: Vec2::new(2., 1.),
            }
        );

        assert_eq!(
            b2,
            Rect {
                min: Vec2::new(-1., -2.),
                max: Vec2::new(1., 2.),
            }
        );
    }

    #[test]
    fn test_calc_bounds_rotate_30() {
        let half_size = Vec2::new(2.0, 1.0);
        let b1 = calc_bounds(Vec2::ZERO, half_size, 30.0f32.to_radians());

        assert_eq!(b1.min, -b1.max);
        assert_eq!(
            b1,
            Rect {
                min: Vec2::new(-2.232051, -1.8660254),
                max: Vec2::new(2.232051, 1.8660254),
            }
        );
    }

    #[test]
    fn test_calc_bounds_rotate_neg_30() {
        let half_size = Vec2::new(2.0, 1.0);
        let b1 = calc_bounds(Vec2::ZERO, half_size, -30.0f32.to_radians());

        assert_eq!(b1.min, -b1.max);
        assert_eq!(
            b1,
            Rect {
                min: Vec2::new(-2.232051, -1.8660254),
                max: Vec2::new(2.232051, 1.8660254),
            }
        );
    }

    #[test]
    fn test_calc_bounds_rotate_120() {
        let half_size = Vec2::new(2.0, 1.0);
        let b1 = calc_bounds(Vec2::ZERO, half_size, 120.0f32.to_radians());

        assert_eq!(b1.min, -b1.max);
        assert_eq!(
            b1,
            Rect {
                min: Vec2::new(-1.8660254, -2.232051),
                max: Vec2::new(1.8660254, 2.232051),
            }
        );
    }

    #[test]
    fn test_calc_bounds_rotate_neg_120() {
        let half_size = Vec2::new(2.0, 1.0);
        let b1 = calc_bounds(Vec2::ZERO, half_size, -120.0f32.to_radians());

        assert_eq!(b1.min, -b1.max);
        assert!(b1.max.y > 2.2);
        assert!(b1.max.x > 1.8);
        assert_eq!(
            b1,
            Rect {
                min: Vec2::new(-1.8660254, -2.232051),
                max: Vec2::new(1.8660254, 2.232051),
            }
        );
    }
}