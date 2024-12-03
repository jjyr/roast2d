use roast2d_derive::Component;

#[derive(Component, Debug, Clone)]
pub struct Health {
    pub max: f32,
    pub value: f32,
    pub killed: bool,
}

impl Health {
    pub fn new(value: f32) -> Self {
        Health {
            value,
            max: value,
            killed: false,
        }
    }

    pub fn percent(&self) -> f32 {
        self.value / self.max
    }

    pub fn is_alive(&self) -> bool {
        self.value > 0.0
    }
}
