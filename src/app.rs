use anyhow::Result;
use glam::UVec2;

use crate::{engine::Engine, platform::platform_run};

#[derive(Debug)]
pub struct App {
    pub title: String,
    pub window: UVec2,
    pub vsync: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            title: "Hello Roast2D".to_string(),
            window: UVec2::new(800, 600),
            vsync: false,
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

    /// Run the game
    pub async fn run<Setup: FnOnce(&mut Engine)>(self, setup: Setup) -> Result<()> {
        let App {
            title,
            window: UVec2 {
                x: width,
                y: height,
            },
            vsync,
        } = self;
        platform_run(title, width, height, vsync, setup).await
    }

    /// Run the game
    #[cfg(not(target_arch = "wasm32"))]
    pub fn run_block<Setup: FnOnce(&mut Engine)>(self, setup: Setup) -> Result<()> {
        futures_lite::future::block_on(self.run(setup))
    }
}
