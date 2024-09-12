use glam::Vec2;

/// Rect
#[derive(Debug, Clone, PartialEq)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    pub(crate) fn is_touching(&self, other: &Self) -> bool {
        !(self.min.x > other.max.x
            || self.max.x < other.min.x
            || self.min.y > other.max.y
            || self.max.y < other.min.y)
    }
}

/// SweepAxis
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SweepAxis {
    #[default]
    X,
    Y,
}

impl SweepAxis {
    pub fn get(self, pos: Vec2) -> f32 {
        match self {
            Self::X => pos.x,
            Self::Y => pos.y,
        }
    }
}
