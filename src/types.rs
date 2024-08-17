use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

pub use glam::Vec2;

/// Rect
#[derive(Debug, Clone)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

/// A mutable reference
pub struct Mut<T: ?Sized>(Rc<RefCell<T>>);

impl<T: ?Sized> From<Rc<RefCell<T>>> for Mut<T> {
    fn from(value: Rc<RefCell<T>>) -> Self {
        Self(value)
    }
}

impl<T> Mut<T> {
    pub fn new(v: T) -> Mut<T> {
        Self(Rc::new(RefCell::new(v)))
    }
}

impl<T: ?Sized> Mut<T> {
    #[cfg_attr(feature = "debug_mut", track_caller)]
    pub fn borrow(&self) -> Ref<'_, T> {
        self.0.borrow()
    }

    #[cfg_attr(feature = "debug_mut", track_caller)]
    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.0.borrow_mut()
    }
}

impl<T: ?Sized> Clone for Mut<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
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
