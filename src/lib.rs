pub mod app;
pub mod asset;
pub mod camera;
pub mod collision;
pub mod collision_map;
pub mod color;
pub mod commands;
pub mod ecs;
pub mod engine;
pub mod entity_hooks;
pub mod font;
pub mod handle;
pub mod health;
pub mod hooks;
pub mod input;
pub mod ldtk;
pub mod map;
pub mod physics;
mod platform;
pub mod prelude;
mod render;
pub mod sat;
pub mod sorts;
pub mod sprite;
pub mod trace;
pub mod transform;
pub mod types;

/// re-export roast2d_derive
pub use roast2d_derive as derive;

/// Alias self crate to allow macro works
extern crate self as roast2d;
