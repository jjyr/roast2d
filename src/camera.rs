use glam::Vec2;

use crate::{
    ecs::{entity::Ent, entity_ref::EntRef},
    transform::Transform,
    types::Rect,
};

#[derive(Default)]
pub struct Camera {
    // A factor of how fast the camera is moving. Values between 0.5..10
    // are usually sensible.
    pub speed: f32,

    // A fixed offset of the screen center from the target entity.
    pub offset: Vec2,

    // Whether to automatically move the bottom of the deadzone up to the
    // target entity when the target is on_ground
    pub snap_to_platform: bool,

    // Whether the followed ent is on the ground
    pub is_followed_ent_on_ground: bool,

    // The minimum velocity (in pixels per second) for a camera movement. If
    // this is set too low and the camera is close to the target it will move
    // very slowly which results in a single pixel movement every few moments,
    // which can look weird. 5 looks good, imho.
    pub min_vel: Vec2,

    // The size of the deadzone: the size of the area around the target within
    // which the camera will not move. The camera will move only when the target
    // is about to leave the deadzone.
    pub deadzone: Vec2,

    // The amount of pixels the camera should be ahead the target. Whether the
    // "ahead" means left/right (or above/below), is determined by the edge of
    // the deadzone that the entity touched last.
    pub look_ahead: Vec2,

    // The top left corner of the viewport. Internally just an offset when
    // drawing background_maps and entities.
    pub(crate) viewport: Vec2,

    // Internal state
    deadzone_pos: Vec2,
    look_ahead_target: Vec2,
    pub(crate) follow: Option<Ent>,
    pos: Vec2,
    vel: Vec2,
    snap: bool,
    force: bool,
}

impl Camera {
    fn viewport_target(&self, screen_size: Vec2, bounds: Option<Vec2>) -> Vec2 {
        let screen_center = screen_size * 0.5;
        let mut viewport_target = self.pos - screen_center + self.offset;
        if let Some(bounds) = bounds {
            viewport_target.x = viewport_target
                .x
                .clamp(0.0, (bounds.x - screen_size.x).max(0.0));
            viewport_target.y = viewport_target
                .y
                .clamp(0.0, (bounds.y - screen_size.y).max(0.0));
        }
        viewport_target
    }

    pub(crate) fn update(
        &mut self,
        tick: f32,
        screen_size: Vec2,
        follow: Option<EntRef>,
        bounds: Option<Vec2>,
    ) {
        if let Some(follow) = follow {
            let Ok(transform) = follow.get::<Transform>() else {
                log::warn!("Camera follow an non transform ent");
                return;
            };
            let follow_size = transform.scaled_size();
            let size = Vec2::new(
                follow_size.x.min(self.deadzone.x),
                follow_size.y.min(self.deadzone.y),
            );
            let half_size = size * 0.5;
            let follow_rect = Rect {
                min: transform.pos - half_size,
                max: transform.pos + half_size,
            };
            if follow_rect.min.x < self.deadzone_pos.x {
                self.deadzone_pos.x = follow_rect.min.x;
                self.look_ahead_target.x = -self.look_ahead.x
            } else if follow_rect.max.x > self.deadzone_pos.x + self.deadzone.x {
                self.deadzone_pos.x = follow_rect.max.x - self.deadzone.x;
                self.look_ahead_target.x = self.look_ahead.x;
            }

            if follow_rect.min.y < self.deadzone_pos.y {
                self.deadzone_pos.y = follow_rect.min.y;
                self.look_ahead_target.y = -self.look_ahead.y;
            } else if follow_rect.max.y > self.deadzone_pos.y + self.deadzone.y {
                self.deadzone_pos.y = follow_rect.max.y - self.deadzone.y;
                self.look_ahead_target.y = self.look_ahead.y;
            }

            if self.snap_to_platform && self.is_followed_ent_on_ground {
                self.deadzone_pos.y = follow_rect.max.y - self.deadzone.y;
            }

            let deadzone_target = self.deadzone_pos + self.deadzone * 0.5;
            self.pos = deadzone_target + self.look_ahead_target;
        }

        if self.force {
            // force set camera
            self.viewport = self.viewport_target(screen_size, bounds);
            self.snap = false;
            self.force = false;
        } else {
            // move camera
            let diff = self.viewport_target(screen_size, bounds) - self.viewport;
            self.vel = diff * self.speed;

            if self.snap
                || self.vel.x.abs() + self.vel.y.abs() > self.min_vel.x.abs() + self.min_vel.y.abs()
            {
                self.viewport += self.vel * tick;
                self.snap = false;
            }
        }
    }

    pub fn pos(&self) -> Vec2 {
        self.pos
    }

    pub fn move_pos(&mut self, pos: Vec2) {
        self.pos = pos;
        self.snap = true;
    }

    pub fn set_pos(&mut self, pos: Vec2) {
        self.pos = pos;
        self.force = true;
    }

    pub fn follow(&mut self, entity_ref: Ent, snap: bool) {
        self.follow.replace(entity_ref);
        self.snap = snap;
    }

    pub fn unfollow(&mut self) {
        self.follow.take();
    }
}
