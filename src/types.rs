use glam::Vec2;

/// Rect
#[derive(Debug, Clone, PartialEq)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    pub fn is_touching(&self, other: &Self) -> bool {
        !(self.min.x > other.max.x
            || self.max.x < other.min.x
            || self.min.y > other.max.y
            || self.max.y < other.min.y)
    }

    pub fn contains_pos(&self, pos: Vec2) -> bool {
        let Rect { min, max } = self;
        pos.x >= min.x && pos.y >= min.y && pos.x <= max.x && pos.y <= max.y
    }
}
