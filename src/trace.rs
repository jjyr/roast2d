use glam::{IVec2, Vec2};

use crate::prelude::CollisionMap;

#[derive(Debug)]
pub struct Trace {
    // The tile that was hit. 0 if no hit.
    pub tile: i32,

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
            tile: 0,
            pos: Vec2::ZERO,
            normal: Vec2::ZERO,
            length: 1.,
            tile_pos: IVec2::ZERO,
        }
    }
}

pub(crate) fn trace(map: &CollisionMap, from_center: Vec2, vel: Vec2, size: Vec2) -> Trace {
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
    let mut extra_step_for_slope = false;
    for i in 0..=(steps as usize) {
        let tile_pos: IVec2 = {
            let p = (corner + step_size * i as f32) / map.tile_size;
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
                check_tile(
                    map,
                    from,
                    vel,
                    size,
                    IVec2::new(tile_pos.x, tile_pos.y + dir.y as i32 * t),
                    &mut res,
                );
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
                check_tile(
                    map,
                    from,
                    vel,
                    size,
                    IVec2::new(tile_pos.x + dir.x as i32 * t, tile_pos.y),
                    &mut res,
                );
            }

            last_tile_pos.y = tile_pos.y;
        }

        // If we collided with a sloped tile, we have to check one more step
        // forward because we may still collide with another tile at an
        // earlier .length point. For fully solid tiles (id: 1), we can
        // return here.
        if res.tile > 0 && (res.tile == 1 || extra_step_for_slope) {
            res.pos += half_size;
            return res;
        }
        extra_step_for_slope = true;
    }

    res.pos += half_size;
    res
}

fn check_tile(
    map: &CollisionMap,
    pos: Vec2,
    vel: Vec2,
    size: Vec2,
    tile_pos: IVec2,
    res: &mut Trace,
) {
    if map.is_collide(tile_pos) {
        resolve_full_tile(map, pos, vel, size, tile_pos, res);
    }
}

fn resolve_full_tile(
    map: &CollisionMap,
    pos: Vec2,
    vel: Vec2,
    size: Vec2,
    tile_pos: IVec2,
    res: &mut Trace,
) {
    // Resolved position, the minimum resulting x or y position in case of a collision.
    // Only the x or y coordinate is correct - depending on if we enter the tile
    // horizontaly or vertically. We will recalculate the wrong one again.

    let mut rp: Vec2 = Vec2::new(
        tile_pos.x as f32 * map.tile_size,
        tile_pos.y as f32 * map.tile_size,
    ) + Vec2::new(
        if vel.x > 0. { -size.x } else { map.tile_size },
        if vel.y > 0. { -size.y } else { map.tile_size },
    );

    // The steps from pos to rp
    let length;

    // If we don't move in Y direction, or we do move in X and the tile
    // corners's cross product with the movement vector has the correct sign,
    // this is a horizontal collision, otherwise it's vertical.
    // float sign = vec2_cross(vel, vec2_sub(rp, pos)) * vel.x * vel.y;
    let sign = (vel.x * (rp.y - pos.y) - vel.y * (rp.x - pos.x)) * vel.x * vel.y;

    if sign < 0. || vel.y == 0. {
        // Horizontal collison (x direction, left or right edge)
        length = ((pos.x - rp.x) / vel.x).abs();
        if length > res.length {
            return;
        };

        rp.y = pos.y + length * vel.y;
        res.normal = Vec2::new(if vel.x > 0.0 { -1.0 } else { 1.0 }, 0.0);
    } else {
        // Vertical collision (y direction, top or bottom edge)
        length = ((pos.y - rp.y) / vel.y).abs();
        if length > res.length {
            return;
        };

        rp.x = pos.x + length * vel.x;
        res.normal = Vec2::new(0.0, if vel.y > 0.0 { -1.0 } else { 1.0 });
    }

    res.tile = 1;
    res.tile_pos = tile_pos;
    res.length = length;
    res.pos = rp;
}
