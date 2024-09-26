use std::any::TypeId;

/// Entity type
#[derive(Hash, PartialEq, Eq, Clone)]
pub struct ComponentId(pub TypeId);

impl ComponentId {
    pub fn of<T: 'static>() -> Self {
        Self(TypeId::of::<T>())
    }

    pub fn is<T: 'static>(&self) -> bool {
        Self::of::<T>().0 == self.0
    }
}

impl From<TypeId> for ComponentId {
    fn from(value: TypeId) -> Self {
        Self(value)
    }
}
pub trait Component {}
