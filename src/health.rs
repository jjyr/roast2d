use crate::ecs::component::Component;

pub struct Health {
    pub health: f32,
    pub alive: bool,
}

impl Component for Health {}
