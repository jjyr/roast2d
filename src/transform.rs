use std::f32::consts::PI;

use glam::Vec2;
use roast2d_derive::Component;

use crate::types::Rect;

#[derive(Component, Debug, Default, Clone)]
pub struct Transform {
    pub pos: Vec2,
    pub scale: Vec2,
    /// Angle in radians
    pub angle: f32,
    pub size: Vec2,
    pub z_index: u32,
}

impl Transform {
    pub fn new(pos: Vec2, size: Vec2) -> Self {
        Self {
            pos,
            size,
            z_index: 0,
            scale: Vec2::splat(1.0),
            angle: 0.0,
        }
    }

    pub fn with_z_index(mut self, z_index: u32) -> Self {
        self.z_index = z_index;
        self
    }

    pub fn scaled_size(&self) -> Vec2 {
        self.size * self.scale
    }

    pub fn bounds(&self) -> Rect {
        let half_size = self.scaled_size() * 0.5;
        calc_bounds(self.pos, half_size, self.angle)
    }
}

pub fn calc_bounds(pos: Vec2, half_size: Vec2, angle: f32) -> Rect {
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
