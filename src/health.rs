use roast2d_derive::Component;

#[derive(Component)]
pub struct Health {
    pub health: f32,
    pub alive: bool,
}
