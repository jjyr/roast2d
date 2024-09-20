use glam::{IVec2, Vec2};

use crate::{
    collision::{calc_overlap, Shape},
    prelude::CollisionMap,
};

#[derive(Debug, Clone)]
pub struct Trace {
    // The tile that was hit. 0 if no hit.
    pub is_collide: bool,

    // The tile position (in tile space) of the hit
    pub tile_pos: IVec2,

    // The normalized 0..1 length of this trace. If this trace did not end in
    // a hit, length will be 1.
    pub length: f32,

    // The resulting position of the top left corne of the AABB that was traced
    pub pos: Vec2,

    // The normal vector of the surface that was hit
    pub normal: Vec2,
}

impl Default for Trace {
    fn default() -> Self {
        Trace {
            is_collide: false,
            pos: Vec2::ZERO,
            normal: Vec2::ZERO,
            length: 1.,
            tile_pos: IVec2::ZERO,
        }
    }
}

pub(crate) fn trace(
    map: &CollisionMap,
    from_center: Vec2,
    vel: Vec2,
    size: Vec2,
    angle: f32,
) -> Trace {
    let half_size = size * 0.5;
    let from = from_center - half_size;
    let to = from + vel;
    let mut res = Trace {
        pos: to,
        ..Default::default()
    };

    // Quick check if the whole trace is out of bounds
    let map_size = map.bounds();
    if (from.x + size.x < 0. && to.x + size.x < 0.)
        || (from.y + size.y < 0. && to.y + size.y < 0.)
        || (from.x > map_size.x && to.x > map_size.x)
        || (from.y > map_size.y && to.y > map_size.y)
        || (vel.x == 0. && vel.y == 0.)
    {
        res.pos += half_size;
        return res;
    }
    let offset = Vec2::new(
        if vel.x > 0. { 1.0 } else { 0.0 },
        if vel.y > 0. { 1.0 } else { 0.0 },
    );

    let corner = from + size * offset;
    let dir = offset * -2.0 + Vec2::splat(1.0);
    let max_vel = (vel.x * -dir.x).max(vel.y * -dir.y);
    let steps = (max_vel / map.tile_size).ceil();

    if steps == 0.0 {
        res.pos += half_size;
        return res;
    }

    let step_size = vel / steps;

    let mut last_tile_pos = IVec2::splat(-16);

    // used to perform sat collision
    let tile_hf_size = Vec2::splat(map.tile_size * 0.5);
    for i in 0..=(steps as usize) {
        let tile_pos: IVec2 = {
            let tile_px = corner + step_size * i as f32;
            let p = tile_px / map.tile_size;
            IVec2::new(p.x as i32, p.y as i32)
        };

        let mut corner_tile_checked = 0;
        if last_tile_pos.x != tile_pos.x {
            // Figure out the number of tiles in Y direction we need to check.
            // This walks along the vertical edge of the object (height) from
            // the current tile_pos.x,tile_pos.y position.
            let mut max_y = from.y + size.y * (1. - offset.y);
            if i > 0 {
                max_y += (vel.y / vel.x)
                    * ((tile_pos.x as f32 + 1. - offset.x) * map.tile_size - corner.x);
            }

            let num_tiles = (max_y / map.tile_size - tile_pos.y as f32 - offset.y)
                .abs()
                .ceil() as i32;
            for t in 0..num_tiles {
                let tile_pos = IVec2::new(tile_pos.x, tile_pos.y + dir.y as i32 * t);
                // check tile collision with sat
                let tile_shape = {
                    let pos = Vec2::new(
                        (tile_pos.x as f32) * map.tile_size + tile_hf_size.x,
                        (tile_pos.y as f32) * map.tile_size + tile_hf_size.y,
                    );

                    Shape {
                        angle: 0.0,
                        half_size: tile_hf_size,
                        pos,
                    }
                };
                let shape = Shape {
                    pos: from_center,
                    angle,
                    half_size,
                };
                if let Some(overlap) = calc_overlap(&tile_shape, &shape) {
                    check_tile(map, from, vel, tile_pos, overlap, &mut res);
                }
            }

            last_tile_pos.x = tile_pos.x;
            corner_tile_checked = 1;
        }

        if last_tile_pos.y != tile_pos.y {
            // Figure out the number of tiles in X direction we need to
            // check. This walks along the horizontal edge of the object
            // (width) from the current tile_pos.x,tile_pos.y position.
            let mut max_x = from.x + size.x * (1. - offset.x);
            if i > 0 {
                max_x += (vel.x / vel.y)
                    * ((tile_pos.y as f32 + 1. - offset.y) * map.tile_size - corner.y);
            }

            let num_tiles = (max_x / map.tile_size - tile_pos.x as f32 - offset.x)
                .abs()
                .ceil() as i32;
            for t in corner_tile_checked..num_tiles {
                let tile_pos = IVec2::new(tile_pos.x + dir.x as i32 * t, tile_pos.y);
                // check tile collision with sat
                let tile_shape = {
                    let pos = Vec2::new(
                        (tile_pos.x as f32) * map.tile_size + tile_hf_size.x,
                        (tile_pos.y as f32) * map.tile_size + tile_hf_size.y,
                    );

                    Shape {
                        angle: 0.0,
                        half_size: tile_hf_size,
                        pos,
                    }
                };
                let shape = Shape {
                    pos: from_center,
                    angle,
                    half_size,
                };
                if let Some(overlap) = calc_overlap(&tile_shape, &shape) {
                    check_tile(map, from, vel, tile_pos, overlap, &mut res);
                }
            }

            last_tile_pos.y = tile_pos.y;
        }

        if res.is_collide {
            break;
        }
    }

    res.pos += half_size;
    res
}

fn check_tile(
    map: &CollisionMap,
    pos: Vec2,
    vel: Vec2,
    tile_pos: IVec2,
    overlap: Vec2,
    res: &mut Trace,
) {
    if map.is_collide(tile_pos) {
        resolve_full_tile(pos, vel, tile_pos, overlap, res);
    }
}

fn resolve_full_tile(pos: Vec2, vel: Vec2, tile_pos: IVec2, overlap: Vec2, res: &mut Trace) {
    // Resolved position, the minimum resulting x or y position in case of a collision.
    // Only the x or y coordinate is correct - depending on if we enter the tile
    // horizontaly or vertically. We will recalculate the wrong one again.

    let rp: Vec2 = pos + overlap;
    res.normal = overlap.normalize_or_zero();

    let length = if overlap.x.abs() > overlap.y.abs() {
        (overlap.x / vel.x).abs()
    } else {
        (overlap.y / vel.y).abs()
    };

    res.is_collide = true;
    res.tile_pos = tile_pos;
    res.length = length;
    res.pos = rp;
}
