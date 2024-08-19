use std::path::Path;

use anyhow::{anyhow, Result};
use glam::{UVec2, Vec2};
use sdl2::{pixels::PixelFormatEnum, rect::Rect, render::Texture, surface::Surface};

use crate::{font::Text, image::Image, platform::ScreenBuffer};

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum ScaleMode {
    #[default]
    None,
    Discrete,
    Exact,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct ResizeMode {
    pub width: bool,
    pub height: bool,
}

/// Render subsystem
pub(crate) struct Render {
    draw_calls: u32,
    screen_scale: f32,
    inv_screen_scale: f32,
    pub(crate) screen_size: Vec2,
    logical_size: Vec2,
    // transform_stack: Vec<Mat3>,
    screen_buffer: Option<ScreenBuffer>,
    scale_mode: ScaleMode,
    resize_mode: ResizeMode,
    view_size: Vec2,
    pub(crate) vsync: bool,
}

impl Default for Render {
    fn default() -> Self {
        Self {
            draw_calls: 0,
            screen_scale: 1.0,
            inv_screen_scale: 1.0,
            screen_size: Vec2::default(),
            logical_size: Vec2::default(),
            // transform_stack: Default::default(),
            screen_buffer: None,
            scale_mode: ScaleMode::default(),
            view_size: Vec2::new(1280.0, 720.0),
            resize_mode: ResizeMode::default(),
            vsync: true,
        }
    }
}

impl Render {
    pub(crate) fn set_screen_buffer(&mut self, screen_buffer: ScreenBuffer) {
        self.screen_buffer.replace(screen_buffer);
    }

    pub(crate) fn screen_buffer_mut(&mut self) -> Option<&mut ScreenBuffer> {
        self.screen_buffer.as_mut()
    }

    pub(crate) fn render_snap_px(&self, pos: Vec2) -> Vec2 {
        let sp = pos * self.screen_scale;
        sp * self.inv_screen_scale
    }

    pub(crate) fn render_draw(
        &mut self,
        mut pos: Vec2,
        mut size: Vec2,
        texture: &Texture,
        uv_offset: Vec2,
        uv_size: Vec2,
        flip_x: bool,
        flip_y: bool,
    ) {
        if pos.x > self.logical_size.x
            || pos.y > self.logical_size.y
            || pos.x + size.x < 0.
            || pos.y + size.y < 0.
        {
            return;
        }

        pos *= self.screen_scale;
        size *= self.screen_scale;
        self.draw_calls += 1;
        if let Some(screen_buffer) = self.screen_buffer.as_mut() {
            let src = Rect::new(
                uv_offset.x as i32,
                uv_offset.y as i32,
                uv_size.x as u32,
                uv_size.y as u32,
            );
            let dst = Rect::new(pos.x as i32, pos.y as i32, size.x as u32, size.y as u32);
            if let Err(err) = screen_buffer
                .canvas
                .copy_ex(texture, src, dst, 0.0, None, flip_x, flip_y)
            {
                eprintln!("SDL render_draw {err}");
            }
        }
    }

    pub(crate) fn resize(&mut self, size: UVec2) {
        // Determine Zoom
        if self.scale_mode == ScaleMode::None {
            self.screen_scale = 1.0;
        } else {
            self.screen_scale =
                (size.x as f32 / self.view_size.x).min(size.y as f32 / self.view_size.y);
            if self.scale_mode == ScaleMode::Discrete {
                self.screen_scale = self.screen_scale.floor().max(0.5);
            }
        }
        // Determine size
        if self.resize_mode.width {
            self.screen_size.x = (size.x as f32).max(self.view_size.x);
        } else {
            self.screen_size.x = self.view_size.x * self.screen_scale;
        }

        if self.resize_mode.height {
            self.screen_size.y = (size.y as f32).max(self.view_size.y);
        } else {
            self.screen_size.y = self.view_size.y * self.screen_scale;
        }

        self.logical_size.x = (self.screen_size.x / self.screen_scale).ceil();
        self.logical_size.y = (self.screen_size.y / self.screen_scale).ceil();
        self.inv_screen_scale = 1.0 / self.screen_scale;
    }

    pub(crate) fn load_image<P: AsRef<Path>>(&self, path: P) -> Result<Image> {
        let im = image::open(path)?;
        let surface = Surface::new(im.width(), im.height(), PixelFormatEnum::ABGR8888)
            .map_err(|err| anyhow!(err))?;
        let pitch = surface.pitch();
        let mut texture = self
            .screen_buffer
            .as_ref()
            .ok_or_else(|| anyhow!("screen buffer"))?
            .texture_creator
            .create_texture_from_surface(surface)?;
        texture.update(None, im.as_bytes(), pitch as usize)?;
        Ok(Image::new(texture))
    }

    pub(crate) fn create_text_texture(&self, text: Text) -> Result<Image> {
        let Text {
            text,
            font,
            scale,
            color,
        } = text;
        let buffer = font.render_text_texture(&text, scale, color);
        let surface = Surface::new(buffer.width(), buffer.height(), PixelFormatEnum::ABGR8888)
            .map_err(|err| anyhow!(err))?;
        let pitch = surface.pitch();
        let screen_buffer = self
            .screen_buffer
            .as_ref()
            .ok_or_else(|| anyhow!("screen buffer"))?;
        let mut texture = screen_buffer
            .texture_creator
            .create_texture_from_surface(surface)?;
        texture.update(None, buffer.as_ref(), pitch as usize)?;
        let mut image = Image::new(texture);
        image.color = color;
        Ok(image)
    }

    pub(crate) fn scale_mode(&self) -> ScaleMode {
        self.scale_mode
    }

    pub(crate) fn set_scale_mode(&mut self, mode: ScaleMode) {
        self.scale_mode = mode;
    }

    pub(crate) fn resize_mode(&self) -> ResizeMode {
        self.resize_mode
    }

    pub(crate) fn set_resize_mode(&mut self, mode: ResizeMode) {
        self.resize_mode = mode;
    }

    pub(crate) fn view_size(&self) -> Vec2 {
        self.view_size
    }

    pub(crate) fn logical_size(&self) -> Vec2 {
        self.logical_size
    }

    pub(crate) fn set_view_size(&mut self, size: Vec2) {
        self.view_size = size;
    }
}
