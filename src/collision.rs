use std::f32::consts::PI;

use glam::Vec2;
use roast2d_derive::Resource;

const ENTITY_MIN_BOUNCE_VELOCITY: f32 = 10.0;

use crate::{
    ecs::{entity::Ent, entity_ref::EntMut, world::World},
    engine::Engine,
    hooks::get_ent_hooks,
    physics::{entity_move, EntCollidesMode, EntPhysics, Physics},
    prelude::Transform,
    sat::{calc_sat_overlap, SatRect},
    sorts::insertion_sort_by_key,
    trace::Trace,
    types::{Rect, SweepAxis},
};

#[derive(Default, Resource)]
pub struct CollisionSet {
    ents: Vec<Ent>,
}

impl CollisionSet {
    pub fn add(&mut self, ent: Ent) {
        self.ents.push(ent);
    }

    pub fn remove(&mut self, ent: Ent) {
        if let Some(index) = self.ents.iter().position(|id| *id == ent) {
            self.ents.remove(index);
        }
    }

    pub(crate) fn sort_entities_for_sweep(&mut self, w: &mut World, sweep_axis: SweepAxis) {
        let mut ents = core::mem::take(&mut self.ents);
        insertion_sort_by_key(&mut ents, |ent| {
            w.get(*ent)
                .and_then(|ent| {
                    ent.get::<Transform>()
                        .map(|t| sweep_axis.get(t.bounds().min) as usize)
                })
                .unwrap_or(usize::MAX)
        });
        let _ = core::mem::replace(&mut self.ents, ents);
    }
}

pub(crate) fn init_collision(_eng: &mut Engine, w: &mut World) {
    w.add_resource(CollisionSet::default());
}

pub(crate) fn update_collision(eng: &mut Engine, w: &mut World) {
    let Some(mut collision_set) = w.remove_resource::<CollisionSet>() else {
        return;
    };

    // Sort by x or y position
    // insertion sort can gain better performance since list is sorted in every frames

    let sweep_axis = eng.sweep_axis;
    collision_set.sort_entities_for_sweep(w, sweep_axis);

    // Sweep touches
    eng.perf.checks = 0;
    let ents_count = collision_set.ents.len();
    for i in 0..ents_count {
        let ent1 = collision_set.ents[i];
        let (res, ent1_bounds) = {
            let Ok(ent1) = w.get(ent1) else {
                continue;
            };
            let Ok(phy1) = ent1.get::<Physics>() else {
                continue;
            };
            let res = !phy1.check_against.is_empty()
                || !phy1.group.is_empty()
                || phy1.physics.is_at_least(EntPhysics::PASSIVE);

            (res, ent1.get::<Transform>().unwrap().bounds())
        };
        if res {
            let max_pos = sweep_axis.get(ent1_bounds.max);
            for j in (i + 1)..ents_count {
                let (ent2, ent2_bounds) = {
                    let ent2 = collision_set.ents[j];
                    let Ok(ent_ref2) = w.get(ent2) else {
                        continue;
                    };
                    let Ok(t2) = ent_ref2.get::<Transform>() else {
                        continue;
                    };
                    let ent2_bounds = t2.bounds();
                    (ent2, ent2_bounds)
                };
                if sweep_axis.get(ent2_bounds.min) > max_pos {
                    break;
                }
                eng.perf.checks += 1;
                if let Some(overlap) = calc_ent_overlap(w, ent1, ent2) {
                    let res = {
                        let [ent1, ent2] = w.many([ent1, ent2]);
                        let Ok(phy1) = ent1.get::<Physics>() else {
                            continue;
                        };
                        let Ok(phy2) = ent2.get::<Physics>() else {
                            continue;
                        };

                        !(phy1.check_against & phy2.group).is_empty()
                    };
                    if res {
                        if let Ok(hook) = get_ent_hooks(w, ent1) {
                            if let Err(err) = hook.touch(eng, w, ent1, ent2) {
                                log::error!(
                                    "Error occuerd on handling touch hook of {ent1:?} {ent2:?}: {err}"
                                );
                            }
                        }
                    }
                    let res = {
                        let [ent1, ent2] = w.many([ent1, ent2]);
                        let Ok(phy1) = ent1.get::<Physics>() else {
                            continue;
                        };
                        let Ok(phy2) = ent2.get::<Physics>() else {
                            continue;
                        };
                        !(phy1.group & phy2.check_against).is_empty()
                    };
                    if res {
                        if let Ok(hook) = get_ent_hooks(w, ent2) {
                            if let Err(err) = hook.touch(eng, w, ent2, ent1) {
                                log::error!(
                                    "Error occuerd on handling touch hook of {ent1:?} {ent2:?}: {err}"
                                );
                            }
                        }
                    }

                    let res = {
                        let [ent1, ent2] = w.many([ent1, ent2]);
                        let Ok(phy1) = ent1.get::<Physics>() else {
                            continue;
                        };
                        let Ok(phy2) = ent2.get::<Physics>() else {
                            continue;
                        };
                        phy1.physics.bits() >= EntCollidesMode::LITE.bits()
                            && phy2.physics.bits() >= EntCollidesMode::LITE.bits()
                            && phy1.physics.bits().saturating_add(phy2.physics.bits())
                                >= (EntCollidesMode::ACTIVE | EntCollidesMode::LITE).bits()
                            && (phy1.mass + phy2.mass) > 0.0
                    };
                    if res {
                        resolve_collision(eng, w, ent1, ent2, overlap);
                    }
                }
            }
        }
    }
    w.add_resource(collision_set);
}

/// Resolve entity collision
pub(crate) fn resolve_collision(eng: &mut Engine, w: &mut World, a: Ent, b: Ent, overlap: Vec2) {
    let [mut a, mut b] = w.many_mut([a, b]);

    let Ok(phy_a) = a.get_mut::<Physics>() else {
        return;
    };
    let Ok(phy_b) = b.get_mut::<Physics>() else {
        return;
    };

    let Vec2 {
        x: overlap_x,
        y: overlap_y,
    } = overlap;

    let a_move;
    let b_move;
    if phy_a.physics.is_collide_mode(EntCollidesMode::LITE)
        || phy_b.physics.is_collide_mode(EntCollidesMode::FIXED)
    {
        a_move = 1.0;
        b_move = 0.0;
    } else if phy_a.physics.is_collide_mode(EntCollidesMode::FIXED)
        || phy_b.physics.is_collide_mode(EntCollidesMode::LITE)
    {
        a_move = 0.0;
        b_move = 1.0;
    } else {
        let total_mass = phy_a.mass + phy_b.mass;
        a_move = phy_b.mass / total_mass;
        b_move = phy_a.mass / total_mass;
    }

    if overlap_y.abs() > overlap_x.abs() {
        if overlap_x > 0.0 {
            entities_separate_on_x_axis(eng, &mut a, &mut b, a_move, b_move, overlap_x.abs());
            eng.collide(a.id(), Vec2::new(-1.0, 0.0), None);
            eng.collide(b.id(), Vec2::new(1.0, 0.0), None);
        } else if overlap_x < 0.0 {
            entities_separate_on_x_axis(eng, &mut b, &mut a, b_move, a_move, overlap_x.abs());
            eng.collide(a.id(), Vec2::new(1.0, 0.0), None);
            eng.collide(b.id(), Vec2::new(-1.0, 0.0), None);
        }
    } else if overlap_y > 0.0 {
        entities_separate_on_y_axis(
            eng,
            &mut a,
            &mut b,
            a_move,
            b_move,
            overlap_y.abs(),
            eng.tick,
        );
        eng.collide(a.id(), Vec2::new(0.0, -1.0), None);
        eng.collide(b.id(), Vec2::new(0.0, 1.0), None);
    } else if overlap_y < 0.0 {
        entities_separate_on_y_axis(
            eng,
            &mut b,
            &mut a,
            b_move,
            a_move,
            overlap_y.abs(),
            eng.tick,
        );
        eng.collide(a.id(), Vec2::new(0.0, 1.0), None);
        eng.collide(b.id(), Vec2::new(0.0, -1.0), None);
    }
}

pub(crate) fn entities_separate_on_x_axis(
    eng: &mut Engine,
    ent_left: &mut EntMut,
    ent_right: &mut EntMut,
    left_move: f32,
    right_move: f32,
    overlap: f32,
) {
    let left = ent_left.get_mut::<Physics>().unwrap();
    let right = ent_right.get_mut::<Physics>().unwrap();
    let impact_velocity = left.vel.x - right.vel.x;
    if left_move > 0.0 {
        left.vel.x = right.vel.x * left_move + left.vel.x * right_move;
        let bounce = impact_velocity * left.restitution;
        if bounce > ENTITY_MIN_BOUNCE_VELOCITY {
            left.vel.x -= bounce;
        }

        entity_move(eng, ent_left, Vec2::new(-overlap * left_move, 0.0));
    }

    if right_move > 0.0 {
        let left = ent_left.get_mut::<Physics>().unwrap();
        let right = ent_right.get_mut::<Physics>().unwrap();
        right.vel.x = left.vel.x * right_move + right.vel.x * left_move;
        let bounce = impact_velocity * right.restitution;
        if bounce > ENTITY_MIN_BOUNCE_VELOCITY {
            right.vel.x += bounce;
        }
        entity_move(eng, ent_right, Vec2::new(overlap * right_move, 0.0));
    }
}

pub(crate) fn entities_separate_on_y_axis(
    eng: &mut Engine,
    ent_top: &mut EntMut,
    ent_bottom: &mut EntMut,
    mut top_move: f32,
    mut bottom_move: f32,
    overlap: f32,
    ticks: f32,
) {
    let top = ent_top.get_mut::<Physics>().unwrap();
    let bottom = ent_bottom.get_mut::<Physics>().unwrap();
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
        entity_move(eng, ent_top, Vec2::new(move_x, -overlap * top_move));
    }

    if bottom_move > 0.0 {
        bottom.vel.y = bottom.vel.y * top_move + top_vel_y * bottom_move;
        let bounce = impact_velocity * bottom.restitution;
        if bounce > ENTITY_MIN_BOUNCE_VELOCITY {
            bottom.vel.y += bounce;
        }
        entity_move(eng, ent_bottom, Vec2::new(0.0, overlap * bottom_move));
    }
}

pub(crate) fn handle_trace_result(eng: &mut Engine, ent: &mut EntMut, t: Trace) {
    if let Ok(transform) = ent.get_mut::<Transform>() {
        transform.pos = t.pos;
    }

    if !t.is_collide {
        return;
    }
    if let Ok(phy) = ent.get_mut::<Physics>() {
        phy.vel = Vec2::ZERO;
    }

    eng.collide(ent.id(), t.normal, Some(t.clone()));

    // If this entity is bouncy, calculate the velocity against the
    // slope's normal (the dot product) and see if we want to bounce
    // back.
    let Ok(phy) = ent.get_mut::<Physics>() else {
        return;
    };
    if phy.restitution > 0. {
        let vel_against_normal = phy.vel.dot(t.normal);

        if vel_against_normal.abs() * phy.restitution > ENTITY_MIN_BOUNCE_VELOCITY {
            let vn = t.normal * vel_against_normal * 2.;
            phy.vel = (phy.vel - vn) * phy.restitution;
            return;
        }
    }

    // If this game has gravity, we may have to set the on_ground flag.
    if (phy.gravity != 0.0) && t.normal.y < -phy.max_ground_normal {
        phy.on_ground = true;

        // If we don't want to slide on slopes, we cheat a bit by
        // fudging the y velocity.
        if t.normal.y < -phy.min_slide_normal {
            phy.vel.y = phy.vel.x * t.normal.x;
        }
    }

    // Rotate the normal vector by 90° ([nx, ny] -> [-ny, nx]) to get
    // the slope vector and calculate the dot product with the velocity.
    // This is the velocity with which we will slide along the slope.
    let rotated_normal = Vec2::new(-t.normal.y, t.normal.x);
    let vel_along_normal = phy.vel.dot(rotated_normal);
    phy.vel = rotated_normal * vel_along_normal;
}

pub(crate) fn calc_bounds(pos: Vec2, half_size: Vec2, angle: f32) -> Rect {
    const HF_PI: f32 = PI * 0.5;

    if angle == 0.0 || angle.abs() == PI {
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
        let p1 = Vec2::new(half_size.x, -half_size.y);
        let p2 = half_size;
        let p3 = Vec2::new(-half_size.x, half_size.y);
        let p4 = -half_size;
        if angle > 0. && angle < HF_PI {
            let max_x = rot.rotate(p1).x;
            let min_x = rot.rotate(p3).x;
            let max_y = rot.rotate(p2).y;
            let min_y = rot.rotate(p4).y;
            Rect {
                min: pos + Vec2::new(min_x, min_y),
                max: pos + Vec2::new(max_x, max_y),
            }
        } else if angle > HF_PI && angle < PI {
            let max_x = rot.rotate(p4).x;
            let min_x = rot.rotate(p2).x;
            let max_y = rot.rotate(p1).y;
            let min_y = rot.rotate(p3).y;
            Rect {
                min: pos + Vec2::new(min_x, min_y),
                max: pos + Vec2::new(max_x, max_y),
            }
        } else if angle > -PI && angle < -HF_PI {
            let max_x = rot.rotate(p3).x;
            let min_x = rot.rotate(p1).x;
            let max_y = rot.rotate(p4).y;
            let min_y = rot.rotate(p2).y;
            Rect {
                min: pos + Vec2::new(min_x, min_y),
                max: pos + Vec2::new(max_x, max_y),
            }
        } else if angle > -HF_PI && angle < 0.0 {
            let max_x = rot.rotate(p2).x;
            let min_x = rot.rotate(p4).x;
            let max_y = rot.rotate(p3).y;
            let min_y = rot.rotate(p1).y;
            Rect {
                min: pos + Vec2::new(min_x, min_y),
                max: pos + Vec2::new(max_x, max_y),
            }
        } else {
            panic!("Unnormalized angle {angle}")
        }
    }
}

pub(crate) fn calc_ent_overlap(w: &mut World, ent1: Ent, ent2: Ent) -> Option<Vec2> {
    let [ent1, ent2] = w.many([ent1, ent2]);
    let t1 = ent1.get::<Transform>().ok()?;
    let t2 = ent2.get::<Transform>().ok()?;
    calc_overlap(
        &Shape {
            pos: t1.pos,
            angle: t1.angle,
            half_size: t1.scaled_size() * 0.5,
        },
        &Shape {
            pos: t2.pos,
            angle: t2.angle,
            half_size: t2.scaled_size() * 0.5,
        },
    )
}

pub(crate) struct Shape {
    pub pos: Vec2,
    pub angle: f32,
    pub half_size: Vec2,
}

pub(crate) fn calc_overlap(s1: &Shape, s2: &Shape) -> Option<Vec2> {
    let b1 = calc_bounds(s1.pos, s1.half_size, s1.angle);
    let b2 = calc_bounds(s2.pos, s2.half_size, s2.angle);
    // check bounds
    if !b1.is_touching(&b2) {
        return None;
    }
    // test if ent is rotated
    if is_right_angle(s1.angle) && is_right_angle(s2.angle) {
        // not rotated, calculate overlap with bounds
        let overlap_x: f32 = if b1.min.x < b2.min.x {
            b1.max.x - b2.min.x
        } else if b2.max.x > b1.min.x {
            -(b2.max.x - b1.min.x)
        } else {
            0.0
        };
        let overlap_y: f32 = if b1.min.y < b2.min.y {
            b1.max.y - b2.min.y
        } else if b2.max.y > b1.min.y {
            -(b2.max.y - b1.min.y)
        } else {
            0.0
        };
        Some(Vec2::new(overlap_x, overlap_y))
    } else {
        // rotated, perform sat check
        let rect1 = SatRect {
            angle: s1.angle,
            pos: s1.pos,
            half_size: s1.half_size,
        };
        let rect2 = SatRect {
            angle: s2.angle,
            pos: s2.pos,
            half_size: s2.half_size,
        };
        calc_sat_overlap(&rect1, &rect2)
    }
}

pub(crate) fn is_right_angle(angle: f32) -> bool {
    if angle == 0.0 {
        return true;
    }
    let a = angle.abs();
    a == PI || a == PI * 0.5
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
        assert_eq!(
            b1,
            Rect {
                min: Vec2::new(-1.8660254, -2.232051),
                max: Vec2::new(1.8660254, 2.232051),
            }
        );
    }

    #[test]
    fn test_calc_bounds_non_center() {
        let half_size = Vec2::new(2.0, 1.0);
        let b1 = calc_bounds(Vec2::splat(100.0), half_size, -120.0f32.to_radians());

        assert_eq!(
            b1,
            Rect {
                min: Vec2::splat(100.0) + Vec2::new(-1.8660254, -2.232051),
                max: Vec2::splat(100.0) + Vec2::new(1.8660254, 2.232051),
            }
        );
    }
}
