use glam::{UVec2, Vec2};

use crate::{color::Color, font::Text, handle::Handle, platform::Platform, sprite::Sprite};

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
    scale_mode: ScaleMode,
    resize_mode: ResizeMode,
    view_size: Vec2,
    pub(crate) platform: Box<dyn Platform + 'static>,
}

impl Render {
    pub(crate) fn new(platform: Box<dyn Platform + 'static>) -> Self {
        Self {
            draw_calls: 0,
            screen_scale: 1.0,
            inv_screen_scale: 1.0,
            screen_size: Vec2::default(),
            logical_size: Vec2::default(),
            scale_mode: ScaleMode::default(),
            view_size: Vec2::new(1280.0, 720.0),
            resize_mode: ResizeMode::default(),
            platform,
        }
    }

    pub(crate) fn snap_px(&self, pos: Vec2) -> Vec2 {
        let sp = pos * self.screen_scale;
        sp * self.inv_screen_scale
    }

    pub(crate) fn draw(
        &mut self,
        handle: &Handle,
        color: Color,
        mut pos: Vec2,
        mut size: Vec2,
        uv_offset: Vec2,
        uv_size: Option<Vec2>,
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

        self.platform.draw(
            handle, color, pos, size, uv_offset, uv_size, 0.0, flip_x, flip_y,
        );
    }

    /// Draw image
    pub fn draw_image(&mut self, image: &Sprite, pos: Vec2) {
        let dst_size = image.sizef() * image.scale;

        // color
        self.draw(
            &image.texture,
            image.color,
            pos,
            dst_size,
            Vec2::ZERO,
            None,
            image.flip_x,
            image.flip_y,
        );
    }

    /// Draw image as tile
    pub fn draw_tile(
        &mut self,
        image: &Sprite,
        tile: u16,
        tile_size: Vec2,
        dst_pos: Vec2,
        flip_x: bool,
        flip_y: bool,
    ) {
        let cols =
            ((image.size().x as f32 - image.padding) / (tile_size.x + image.spacing)).ceil() as u32;
        let row = tile as u32 / cols;
        let col = tile as u32 % cols;
        let src_pos = Vec2::new(
            col as f32 * (tile_size.x + image.spacing) + image.padding,
            row as f32 * (tile_size.y + image.spacing) + image.padding,
        );
        let src_size = Vec2::new(tile_size.x, tile_size.y);
        let dst_size = src_size * image.scale;

        // color
        let flip_x = flip_x || image.flip_x;
        let flip_y = flip_y || image.flip_y;
        self.draw(
            &image.texture,
            image.color,
            dst_pos,
            dst_size,
            src_pos,
            Some(src_size),
            flip_x,
            flip_y,
        );
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

    pub(crate) fn create_text_texture(&mut self, handle: Handle, text: Text) -> UVec2 {
        let Text {
            text,
            font,
            scale,
            color,
        } = text;
        let buffer = font.render_text_texture(&text, scale, color);
        let width = buffer.width();
        let height = buffer.height();
        let size = UVec2::new(width, height);
        self.platform
            .create_texture(handle, buffer.into_vec(), size);
        size
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
