//! Separating axis theorem

use glam::Vec2;

#[derive(Debug)]
pub struct SatRect {
    pub pos: Vec2,       // center
    pub half_size: Vec2, // half size
    pub angle: f32,      // angle in radians
}

impl SatRect {
    pub fn get_vertices(&self) -> [Vec2; 4] {
        let (s, c) = self.angle.sin_cos();
        let Vec2 { x: w, y: h } = self.half_size;

        // Get rotated vertex
        [
            Vec2 {
                x: -w * c - h * s,
                y: -w * s + h * c,
            }, // Left bottom
            Vec2 {
                x: w * c - h * s,
                y: w * s + h * c,
            }, // Right bottom
            Vec2 {
                x: w * c + h * s,
                y: w * s - h * c,
            }, // Right top
            Vec2 {
                x: -w * c + h * s,
                y: -w * s - h * c,
            }, // Left top
        ]
        .map(|v| self.pos + v)
    }
}

// 2D Projection
#[derive(Debug)]
struct Projection {
    min: f32,
    max: f32,
}

fn project(vertices: &[Vec2], axis: Vec2) -> Projection {
    let mut min = vertices[0].dot(axis);
    let mut max = min;

    for v in &vertices[1..] {
        let p = v.dot(axis);
        if p < min {
            min = p;
        } else if p > max {
            max = p;
        }
    }

    Projection { min, max }
}

// calculate overlap
fn calc_overlap(proj1: Projection, proj2: Projection) -> Option<f32> {
    let overlap = f32::min(proj1.max, proj2.max) - f32::max(proj1.min, proj2.min);
    if overlap > 0.0 {
        Some(overlap)
    } else {
        None
    }
}

// SAT collision overlap
pub fn sat_collision_overlap(rect1: &SatRect, rect2: &SatRect) -> Option<Vec2> {
    let vs1 = rect1.get_vertices();
    let vs2 = rect2.get_vertices();

    // we only need to check two axes per rect
    let axes = [
        (vs1[1] - vs1[0]).perp(), // rect1
        (vs1[3] - vs1[0]).perp(), // rect1
        (vs2[1] - vs2[0]).perp(), // rect2
        (vs2[3] - vs2[0]).perp(), // rect2
    ];

    let mut min_overlap = f32::MAX;
    let mut min_axis = Vec2 { x: 0.0, y: 0.0 };

    for axis in &axes {
        let axis = axis.normalize();
        let proj1 = project(&vs1, axis);
        let proj2 = project(&vs2, axis);

        // calculate overlap
        if let Some(overlap) = calc_overlap(proj1, proj2) {
            if overlap < min_overlap {
                min_overlap = overlap;
                min_axis = axis;
            }
        } else {
            // no collision
            return None;
        }
    }

    // return overlap on x, y
    Some(Vec2 {
        x: min_axis.x * min_overlap,
        y: min_axis.y * min_overlap,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sat_collide() {
        let rect1 = SatRect {
            pos: Vec2 { x: 0.0, y: 0.0 },
            half_size: Vec2 { x: 50.0, y: 30.0 },
            angle: 0.0,
        };

        let rect2 = SatRect {
            pos: Vec2 { x: 70.0, y: 50.0 },
            half_size: Vec2 { x: 30.0, y: 20.0 },
            angle: 0.5,
        };

        let overlap = sat_collision_overlap(&rect1, &rect2).expect("overlap");
        assert_eq!(overlap, Vec2::new(2.5097463, 1.3710803));
    }

    #[test]
    fn test_not_sat_collide() {
        let rect1 = SatRect {
            pos: Vec2 { x: 0.0, y: 0.0 },
            half_size: Vec2 { x: 50.0, y: 30.0 },
            angle: 0.5,
        };

        let rect2 = SatRect {
            pos: Vec2 { x: 70.0, y: 50.0 },
            half_size: Vec2 { x: 30.0, y: 20.0 },
            angle: 0.5,
        };

        let overlap = sat_collision_overlap(&rect1, &rect2);
        assert_eq!(overlap, None);
    }
}
