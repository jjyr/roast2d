use anyhow::Result;
use glam::UVec2;

use crate::{ecs::world::World, engine::Engine, platform::platform_run};

#[derive(Debug)]
pub struct App {
    pub title: String,
    pub window: UVec2,
    pub vsync: bool,
    pub resizable: bool,
    pub fullscreen: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            title: "Hello Roast2D".to_string(),
            window: UVec2::new(800, 600),
            vsync: false,
            resizable: false,
            fullscreen: false,
        }
    }
}

impl App {
    pub fn title(mut self, title: String) -> Self {
        self.title = title;
        self
    }

    pub fn window(mut self, window: UVec2) -> Self {
        self.window = window;
        self
    }

    pub fn vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn fullscreen(mut self, fullscreen: bool) -> Self {
        self.fullscreen = fullscreen;
        self
    }

    /// Run the game
    pub async fn run<Setup: FnOnce(&mut Engine, &mut World)>(self, setup: Setup) -> Result<()> {
        platform_run(self, setup).await
    }
}
