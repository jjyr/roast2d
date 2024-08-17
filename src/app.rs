use anyhow::Result;

use crate::{engine::Engine, platform};

/// Run the game
pub fn run(mut engine: Engine, title: String, width: u32, height: u32) -> Result<()> {
    platform::sdl::init(&mut engine, title, width, height)
}
