use crate::sprite::Sprite;

#[derive(Clone)]
pub struct Animation {
    pub sheet: Sprite,
}

impl Animation {
    pub fn new(sheet: Sprite) -> Self {
        Self { sheet }
    }
}
