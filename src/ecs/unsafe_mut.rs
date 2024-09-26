use std::cell::UnsafeCell;

/// A mutable reference
pub(crate) struct Mut<T: ?Sized>(UnsafeCell<T>);

impl<T> Mut<T> {
    pub(crate) fn new(v: T) -> Mut<T> {
        Self(UnsafeCell::new(v))
    }
}

impl<T: ?Sized> Mut<T> {
    pub(crate) unsafe fn get(&self) -> &T {
        self.0.get().as_ref().unwrap()
    }

    #[allow(clippy::mut_from_ref)]
    pub(crate) unsafe fn get_mut(&self) -> &mut T {
        self.0.get().as_mut().unwrap()
    }
}
