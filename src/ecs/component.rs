use std::any::{Any, TypeId};

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
pub trait Component {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}
