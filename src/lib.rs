pub mod app;
pub mod asset;
pub mod camera;
pub mod color;
pub mod ecs;
pub mod engine;
pub mod errors;
pub mod font;
pub mod handle;
pub mod health;
pub mod input;
pub mod map;
mod platform;
pub mod prelude;
mod render;
pub mod sat;
pub mod sprite;
pub mod text_cache;
pub mod transform;
pub mod types;

/// re-export roast2d_derive
pub use roast2d_derive as derive;

/// Alias self crate to allow macro works
extern crate self as roast2d;
