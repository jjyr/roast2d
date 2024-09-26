//! UnsafeWorldRef
//! A usafe reference can access world,
//! this implementation is a simplified version of bevy unsafe world cell
//! https://github.com/bevyengine/bevy/blob/main/crates/bevy_ecs/src/world/unsafe_world_cell.rs
use std::{marker::PhantomData, ptr};

use super::world::World;

/// Unsafe world ref
pub struct UnsafeWorldRef<'w>(pub(crate) *mut World, PhantomData<&'w World>);

impl<'w> UnsafeWorldRef<'w> {
    #[inline]
    pub(crate) fn new_readonly(world: &'w World) -> Self {
        Self(ptr::from_ref(world).cast_mut(), PhantomData)
    }

    #[inline]
    pub(crate) fn new_mutable(world: &'w mut World) -> Self {
        Self(ptr::from_mut(world), PhantomData)
    }

    /// # Safety
    ///
    /// Used to get world reference
    /// This function is safe since we require lifetime
    pub unsafe fn as_ref(&self) -> &World {
        self.0.as_ref().unwrap()
    }

    /// # Safety
    ///
    /// Used to get world mut
    /// This function is safe since we require lifetime
    pub unsafe fn as_mut(&mut self) -> &mut World {
        self.0.as_mut().unwrap()
    }
}
